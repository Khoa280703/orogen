#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from collections import Counter
from pathlib import Path
from typing import Any
from urllib.parse import urlparse


def load_manifest(path: Path) -> dict[str, Any]:
    try:
        payload = json.loads(path.read_text())
    except FileNotFoundError as exc:
        raise SystemExit(f"Manifest not found: {path}") from exc
    except json.JSONDecodeError as exc:
        raise SystemExit(f"Manifest contains invalid JSON: {exc}") from exc
    if not isinstance(payload, dict):
        raise SystemExit("Manifest must be a JSON object.")
    return payload


def ensure(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def validate_url(raw_url: str) -> None:
    parsed = urlparse(raw_url)
    ensure(raw_url == raw_url.strip(), f"URL has leading or trailing whitespace: {raw_url!r}")
    ensure(parsed.scheme in {"http", "https"}, f"Invalid URL scheme: {raw_url}")
    ensure(bool(parsed.netloc), f"Invalid URL host: {raw_url}")
    ensure(parsed.username is None and parsed.password is None, f"URL must not contain credentials: {raw_url}")
    ensure(" " not in raw_url, f"URL must not contain spaces: {raw_url}")


def resolve_patterns(base_dir: Path, patterns: list[str]) -> set[Path]:
    matches: set[Path] = set()
    for pattern in patterns:
        matches.update(path.resolve() for path in base_dir.glob(pattern))
    return matches


def summarize_manifest(manifest_path: Path, manifest: dict[str, Any]) -> dict[str, Any]:
    ensure(manifest.get("format") == "qwen35-continual-pretraining-corpus-v1", "Unsupported manifest format.")
    ensure(isinstance(manifest.get("model"), str) and manifest["model"], "Manifest is missing model.")
    ensure(isinstance(manifest.get("round_id"), str) and manifest["round_id"], "Manifest is missing round_id.")
    ensure(isinstance(manifest.get("manifest_stage"), str) and manifest["manifest_stage"], "Manifest is missing manifest_stage.")
    ensure(isinstance(manifest.get("language"), str) and manifest["language"], "Manifest is missing language.")
    ensure(isinstance(manifest.get("description"), str) and manifest["description"], "Manifest is missing description.")
    ensure(isinstance(manifest.get("target_tokens"), int) and manifest["target_tokens"] > 0, "Invalid target_tokens.")
    ensure(isinstance(manifest.get("dedupe_policy"), dict), "Manifest is missing dedupe_policy.")
    ensure(isinstance(manifest.get("sanitization_policy"), dict), "Manifest is missing sanitization_policy.")
    ensure(isinstance(manifest.get("sources"), list) and manifest["sources"], "Manifest must contain sources.")

    base_dir = manifest_path.parent
    seen_ids: set[str] = set()
    kind_counts: Counter[str] = Counter()
    domain_counts: Counter[str] = Counter()
    status_counts: Counter[str] = Counter()
    stage_counts: Counter[str] = Counter()
    warnings: list[str] = []
    source_summaries: list[dict[str, Any]] = []
    local_path_owners: dict[Path, set[str]] = {}
    weight_sum = 0.0
    token_sum = 0

    for source in manifest["sources"]:
        ensure(isinstance(source, dict), "Each source must be an object.")
        source_id = source.get("id")
        ensure(isinstance(source_id, str) and source_id, "Each source must have a non-empty id.")
        ensure(source_id not in seen_ids, f"Duplicate source id: {source_id}")
        seen_ids.add(source_id)

        kind = source.get("kind")
        domain = source.get("domain")
        stage = source.get("stage")
        status = source.get("commercial_status")
        weight = source.get("sampling_weight")
        tokens = source.get("estimated_tokens")
        provenance = source.get("provenance")
        filters = source.get("filters")
        ensure(isinstance(kind, str) and kind, f"{source_id}: invalid kind")
        ensure(isinstance(domain, str) and domain, f"{source_id}: invalid domain")
        ensure(isinstance(stage, str) and stage, f"{source_id}: invalid stage")
        ensure(isinstance(status, str) and status, f"{source_id}: invalid commercial_status")
        ensure(isinstance(weight, (int, float)) and weight > 0, f"{source_id}: invalid sampling_weight")
        ensure(isinstance(tokens, int) and tokens > 0, f"{source_id}: invalid estimated_tokens")
        ensure(isinstance(filters, dict), f"{source_id}: filters must be an object")
        ensure(isinstance(provenance, dict), f"{source_id}: provenance must be an object")
        ensure(isinstance(provenance.get("type"), str) and provenance["type"], f"{source_id}: invalid provenance.type")
        ensure(isinstance(provenance.get("reference"), str) and provenance["reference"], f"{source_id}: invalid provenance.reference")
        ensure(isinstance(provenance.get("ready"), bool), f"{source_id}: provenance.ready must be a boolean")

        source_summary = {
            "id": source_id,
            "kind": kind,
            "domain": domain,
            "stage": stage,
            "commercial_status": status,
            "sampling_weight": round(float(weight), 6),
            "estimated_tokens": tokens,
            "provenance": {
                "type": provenance["type"],
                "reference": provenance["reference"],
                "ready": provenance["ready"],
            },
        }

        if kind == "local-glob":
            include_globs = source.get("include_globs")
            exclude_globs = source.get("exclude_globs", [])
            minimum_matches = source.get("minimum_local_matches")
            ensure(isinstance(include_globs, list) and include_globs, f"{source_id}: local-glob source needs include_globs")
            ensure(isinstance(exclude_globs, list), f"{source_id}: exclude_globs must be a list")
            ensure(isinstance(minimum_matches, int) and minimum_matches > 0, f"{source_id}: invalid minimum_local_matches")
            included = resolve_patterns(base_dir, include_globs)
            excluded = resolve_patterns(base_dir, exclude_globs)
            matched = sorted(path for path in included if path not in excluded and path.is_file())
            ensure(
                len(matched) >= minimum_matches,
                f"{source_id}: matched {len(matched)} files, expected at least {minimum_matches}",
            )
            for path in matched:
                owners = local_path_owners.setdefault(path, set())
                owners.add(source_id)
            source_summary["local_scan"] = {
                "matched_files": len(matched),
                "minimum_required_matches": minimum_matches,
                "sample_paths": [str(path) for path in matched[:5]],
            }
        else:
            uris = source.get("uris")
            ensure(isinstance(uris, list) and uris, f"{source_id}: non-local source needs uris")
            ensure(len(set(uris)) == len(uris), f"{source_id}: duplicate URIs are not allowed")
            for raw_url in uris:
                ensure(isinstance(raw_url, str) and raw_url, f"{source_id}: uri must be a non-empty string")
                validate_url(raw_url)
            source_summary["uri_count"] = len(uris)

        kind_counts[kind] += 1
        domain_counts[domain] += 1
        status_counts[status] += 1
        stage_counts[stage] += 1
        weight_sum += float(weight)
        token_sum += tokens
        source_summaries.append(source_summary)

    overlapping_paths = [
        {"path": str(path), "sources": sorted(owners)}
        for path, owners in sorted(local_path_owners.items())
        if len(owners) > 1
    ]
    if overlapping_paths:
        warnings.append(f"Detected {len(overlapping_paths)} overlapping local files across sources.")
    ensure(abs(weight_sum - 1.0) <= 1e-6, f"sampling_weight sum is {weight_sum:.6f}, expected 1.0")
    ensure(token_sum == manifest["target_tokens"], f"estimated_tokens sum is {token_sum}, expected {manifest['target_tokens']}")
    if status_counts.get("review-required", 0):
        warnings.append("Some public sources require legal/licensing review before commercial use.")
    pending_provenance_sources = [
        source["id"]
        for source in source_summaries
        if not source["provenance"]["ready"]
    ]
    if pending_provenance_sources:
        warnings.append(
            "Some sources still need pinned provenance before fetch/train: "
            + ", ".join(sorted(pending_provenance_sources))
        )

    return {
        "ok": True,
        "manifest": {
            "path": str(manifest_path),
            "format": manifest["format"],
            "model": manifest["model"],
            "round_id": manifest["round_id"],
            "manifest_stage": manifest["manifest_stage"],
            "language": manifest["language"],
            "target_tokens": manifest["target_tokens"],
            "source_count": len(source_summaries),
        },
        "summary": {
            "sampling_weight_sum": round(weight_sum, 6),
            "estimated_tokens_sum": token_sum,
            "sources_by_kind": dict(sorted(kind_counts.items())),
            "sources_by_domain": dict(sorted(domain_counts.items())),
            "sources_by_stage": dict(sorted(stage_counts.items())),
            "sources_by_commercial_status": dict(sorted(status_counts.items())),
            "sources_with_ready_provenance": len(source_summaries) - len(pending_provenance_sources),
            "sources_pending_provenance": len(pending_provenance_sources),
        },
        "sources": source_summaries,
        "overlapping_local_paths": overlapping_paths,
        "warnings": warnings,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--manifest",
        default="corpora/qwen35-continual-pretraining-round-1-corpus-manifest.json",
    )
    parser.add_argument("--output", default="")
    args = parser.parse_args()

    manifest_path = Path(args.manifest).resolve()
    manifest = load_manifest(manifest_path)
    summary = summarize_manifest(manifest_path, manifest)
    output = json.dumps(summary, ensure_ascii=False, indent=2) + "\n"
    if args.output:
        output_path = Path(args.output).resolve()
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(output)
    print(output, end="")


if __name__ == "__main__":
    main()
