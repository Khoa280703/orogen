#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from collections import Counter
from pathlib import Path

from qwen35_cpt_fetch_utils import ensure, load_manifest
from qwen35_github_issue_fetcher import fetch_issue_documents


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", default="output/qwen35-external-cpt-pinned-manifest-candidate.json")
    parser.add_argument("--source-id", default="issue-fix-and-troubleshooting-narratives")
    parser.add_argument("--output-jsonl", default="output/qwen35-issue-cpt-candidate-shard.jsonl")
    parser.add_argument("--output-summary", default="output/qwen35-issue-cpt-candidate-summary.json")
    parser.add_argument("--max-issues-per-uri", type=int, default=8)
    args = parser.parse_args()

    manifest_path = Path(args.manifest).resolve()
    output_jsonl = Path(args.output_jsonl).resolve()
    output_summary = Path(args.output_summary).resolve()
    output_jsonl.parent.mkdir(parents=True, exist_ok=True)
    output_summary.parent.mkdir(parents=True, exist_ok=True)

    manifest = load_manifest(manifest_path)
    ensure(manifest.get("format") == "qwen35-continual-pretraining-corpus-v1", "Unsupported manifest format.")
    source = next((item for item in manifest.get("sources", []) if item.get("id") == args.source_id), None)
    ensure(source is not None, f"Source not found: {args.source_id}")
    ensure(source.get("kind") == "issue-archive", f"{args.source_id}: expected issue-archive kind")

    source = dict(source)
    source["round_id"] = manifest["round_id"]
    documents: list[dict] = []
    seen_hashes: set[str] = set()
    warnings: list[str] = []
    uri_stats: list[dict[str, object]] = []
    totals = Counter()

    for uri in source.get("uris", []):
        try:
            fetched, stats = fetch_issue_documents(source, uri, args.max_issues_per_uri)
        except SystemExit:
            raise
        except Exception as exc:  # noqa: BLE001
            warnings.append(f"{source['id']}: failed to fetch candidate issues from {uri}: {type(exc).__name__}")
            totals["failed_uris"] += 1
            continue
        uri_stat = {"seed_uri": uri, **stats}
        uri_stats.append(uri_stat)
        for document in fetched:
            digest = document["content_sha256"]
            if digest in seen_hashes:
                totals["deduped_documents"] += 1
                continue
            seen_hashes.add(digest)
            documents.append(document)
            totals["documents"] += 1

    with output_jsonl.open("w", encoding="utf-8") as handle:
        for document in documents:
            handle.write(json.dumps(document, ensure_ascii=False) + "\n")

    summary = {
        "ok": True,
        "manifest": str(manifest_path),
        "source_id": source["id"],
        "output_jsonl": str(output_jsonl),
        "document_count": len(documents),
        "uri_stats": uri_stats,
        "stats": {
            "deduped_documents": totals["deduped_documents"],
            "failed_uris": totals["failed_uris"],
        },
        "warnings": warnings,
    }
    output_summary.write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(summary, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
