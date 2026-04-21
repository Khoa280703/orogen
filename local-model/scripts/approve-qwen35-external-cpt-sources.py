#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

from qwen35_cpt_fetch_utils import ensure, load_manifest


SUPPORTED_KINDS = {"mixed-archive", "doc-archive"}


def load_source_ids(path: str) -> list[str]:
    if not path:
        return []
    raw = Path(path).read_text().splitlines()
    return [line.strip() for line in raw if line.strip() and not line.lstrip().startswith("#")]


def strong_ref_count(source: dict[str, Any]) -> int:
    resolved = source.get("provenance", {}).get("resolved_references", [])
    strong = 0
    for ref in resolved:
        if " -> git:" in ref or "#etag=" in ref or "#last-modified=" in ref:
            strong += 1
    return strong


def weak_ref_count(source: dict[str, Any]) -> int:
    resolved = source.get("provenance", {}).get("resolved_references", [])
    return max(len(resolved) - strong_ref_count(source), 0)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", default="output/qwen35-external-cpt-pinned-manifest-candidate.json")
    parser.add_argument("--output-manifest", default="output/qwen35-external-cpt-approved-manifest.json")
    parser.add_argument("--output-summary", default="output/qwen35-external-cpt-approval-summary.json")
    parser.add_argument("--approve-source", action="append", default=[])
    parser.add_argument("--approve-list", default="")
    parser.add_argument("--approve-all-supported", action="store_true")
    parser.add_argument("--allow-weak-ref-source", action="append", default=[])
    args = parser.parse_args()

    manifest_path = Path(args.manifest).resolve()
    output_manifest = Path(args.output_manifest).resolve()
    output_summary = Path(args.output_summary).resolve()
    output_manifest.parent.mkdir(parents=True, exist_ok=True)
    output_summary.parent.mkdir(parents=True, exist_ok=True)

    manifest = load_manifest(manifest_path)
    ensure(manifest.get("format") == "qwen35-continual-pretraining-corpus-v1", "Unsupported manifest format.")
    approved_manifest = json.loads(json.dumps(manifest))

    requested_ids = set(args.approve_source)
    requested_ids.update(load_source_ids(args.approve_list))
    public_sources = [
        source for source in approved_manifest.get("sources", [])
        if source.get("provenance", {}).get("type") == "public"
    ]
    available_ids = {source["id"] for source in public_sources}
    unsupported_requests = sorted(requested_ids - available_ids)
    ensure(not unsupported_requests, f"Unknown public source ids: {', '.join(unsupported_requests)}")

    approved_ids: list[str] = []
    pending_ids: list[str] = []
    warnings: list[str] = []
    allow_weak = set(args.allow_weak_ref_source)

    for source in public_sources:
        source_id = source["id"]
        should_approve = args.approve_all_supported or source_id in requested_ids
        resolved_refs = source.get("provenance", {}).get("resolved_references", [])
        if should_approve:
            ensure(source.get("kind") in SUPPORTED_KINDS, f"{source_id}: kind {source.get('kind')} is not supported for approved fetch")
            ensure(resolved_refs, f"{source_id}: missing resolved_references from metadata lane")
            ensure(len(resolved_refs) == len(source.get("uris", [])), f"{source_id}: resolved reference count does not match URI count")
            weak_refs = weak_ref_count(source)
            ensure(
                weak_refs == 0 or source_id in allow_weak,
                f"{source_id}: contains {weak_refs} weak references; approve explicitly with --allow-weak-ref-source",
            )
            source["commercial_status"] = "approved-for-fetch"
            source["provenance"]["ready"] = True
            source["provenance"]["approval"] = {
                "status": "approved-for-fetch",
                "source_manifest": str(manifest_path),
                "weak_reference_count": weak_refs,
            }
            approved_ids.append(source_id)
            if weak_refs:
                warnings.append(f"{source_id}: approved with {weak_refs} weak references")
        else:
            pending_ids.append(source_id)

    summary = {
        "ok": True,
        "manifest": str(manifest_path),
        "output_manifest": str(output_manifest),
        "approved_source_count": len(approved_ids),
        "pending_source_count": len(pending_ids),
        "approved_sources": approved_ids,
        "pending_sources": pending_ids,
        "warnings": warnings,
    }

    output_manifest.write_text(json.dumps(approved_manifest, ensure_ascii=False, indent=2) + "\n")
    output_summary.write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(summary, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
