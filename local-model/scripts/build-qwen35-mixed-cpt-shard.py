#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from collections import Counter
from pathlib import Path


def read_jsonl(path: Path) -> list[dict]:
    try:
        lines = path.read_text().splitlines()
    except FileNotFoundError as exc:
        raise SystemExit(f"Input JSONL not found: {path}") from exc
    documents = [json.loads(line) for line in lines if line.strip()]
    if not all(isinstance(document, dict) for document in documents):
        raise SystemExit(f"Input JSONL contains non-object rows: {path}")
    return documents


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--local-jsonl", default="output/qwen35-local-cpt-shard.jsonl")
    parser.add_argument("--external-jsonl", default="output/qwen35-external-cpt-shard.jsonl")
    parser.add_argument("--output-jsonl", default="output/qwen35-mixed-cpt-shard.jsonl")
    parser.add_argument("--output-summary", default="output/qwen35-mixed-cpt-shard-summary.json")
    args = parser.parse_args()

    local_path = Path(args.local_jsonl).resolve()
    external_path = Path(args.external_jsonl).resolve()
    output_jsonl = Path(args.output_jsonl).resolve()
    output_summary = Path(args.output_summary).resolve()
    output_jsonl.parent.mkdir(parents=True, exist_ok=True)
    output_summary.parent.mkdir(parents=True, exist_ok=True)

    merged: list[dict] = []
    seen_hashes: set[str] = set()
    source_counts: Counter[str] = Counter()
    domain_counts: Counter[str] = Counter()
    stats = Counter()

    for label, path in [("local", local_path), ("external", external_path)]:
        for document in read_jsonl(path):
            digest = document.get("content_sha256")
            if not isinstance(digest, str) or not digest:
                raise SystemExit(f"{label}: missing content_sha256")
            if digest in seen_hashes:
                stats[f"deduped_{label}"] += 1
                continue
            seen_hashes.add(digest)
            merged.append(document)
            source_counts[document.get("source_id", "unknown")] += 1
            domain_counts[document.get("domain", "unknown")] += 1

    with output_jsonl.open("w", encoding="utf-8") as handle:
        for document in merged:
            handle.write(json.dumps(document, ensure_ascii=False) + "\n")

    summary = {
        "ok": True,
        "local_jsonl": str(local_path),
        "external_jsonl": str(external_path),
        "output_jsonl": str(output_jsonl),
        "document_count": len(merged),
        "sources_included": dict(sorted(source_counts.items())),
        "domains_included": dict(sorted(domain_counts.items())),
        "stats": dict(sorted(stats.items())),
    }
    output_summary.write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(summary, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
