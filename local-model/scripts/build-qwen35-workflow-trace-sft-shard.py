#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from collections import Counter
from pathlib import Path
from typing import Any

from datasets import load_dataset

from qwen35_workflow_trace_sft_manifest_utils import load_manifest, summarize_manifest
from qwen35_workflow_trace_sft_shard_utils import build_example_record, load_source_runtime_targets, normalize_messages


def resolve_runtime_target(source: dict[str, Any], metadata_index: dict[str, dict[str, Any]]) -> tuple[str | None, str]:
    metadata = metadata_index.get(source["id"], {})
    config_name = metadata.get("selected_config")
    if not isinstance(config_name, str):
        preferred = source.get("preferred_configs", [])
        config_name = preferred[0] if preferred else "default"
    split_name = metadata.get("selected_split")
    if not isinstance(split_name, str):
        preferred = source.get("preferred_splits", [])
        split_name = preferred[0] if preferred else "train"
    if config_name == "default":
        return None, split_name
    return config_name, split_name


def load_stream(source: dict[str, Any], config_name: str | None, split_name: str, shuffle_buffer_size: int):
    dataset = load_dataset(source["hf_dataset"], name=config_name, split=split_name, streaming=True)
    if shuffle_buffer_size > 1:
        dataset = dataset.shuffle(seed=42, buffer_size=shuffle_buffer_size)
    return dataset


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", default="corpora/qwen35-workflow-trace-sft-round-1-manifest.json")
    parser.add_argument("--metadata", default="output/qwen35-workflow-trace-sft-source-metadata.json")
    parser.add_argument("--output-jsonl", default="output/qwen35-workflow-trace-sft-shard.jsonl")
    parser.add_argument("--output-summary", default="output/qwen35-workflow-trace-sft-shard-summary.json")
    parser.add_argument("--shuffle-buffer-size", type=int, default=10000)
    args = parser.parse_args()

    manifest_path = Path(args.manifest).resolve()
    metadata_path = Path(args.metadata).resolve()
    output_jsonl = Path(args.output_jsonl).resolve()
    output_summary = Path(args.output_summary).resolve()
    output_jsonl.parent.mkdir(parents=True, exist_ok=True)
    output_summary.parent.mkdir(parents=True, exist_ok=True)

    manifest = load_manifest(manifest_path)
    manifest_summary = summarize_manifest(manifest_path, manifest)
    metadata_index = load_source_runtime_targets(metadata_path)

    records: list[dict[str, Any]] = []
    seen_hashes: set[str] = set()
    group_counts: Counter[str] = Counter()
    source_counts: Counter[str] = Counter()
    stats: Counter[str] = Counter()
    source_summaries: list[dict[str, Any]] = []
    warnings = list(manifest_summary["warnings"])

    for source in manifest["sources"]:
        config_name, split_name = resolve_runtime_target(source, metadata_index)
        source_written = 0
        source_deduped = 0
        source_invalid = 0
        source_reasoning = 0
        target_examples = source["max_examples"]
        stream = load_stream(source, config_name, split_name, max(1, args.shuffle_buffer_size))

        for row_index, row in enumerate(stream):
            if source_written >= target_examples:
                break
            try:
                messages, row_metadata = normalize_messages(row, source)
                record = build_example_record(
                    manifest,
                    source,
                    config_name or "default",
                    split_name,
                    row_index,
                    messages,
                    row_metadata,
                )
            except Exception:  # noqa: BLE001
                source_invalid += 1
                stats["invalid_examples"] += 1
                continue
            digest = record["normalized_sha256"]
            if digest in seen_hashes:
                source_deduped += 1
                stats["deduped_examples"] += 1
                continue
            seen_hashes.add(digest)
            records.append(record)
            source_written += 1
            source_counts[source["id"]] += 1
            group_counts[source["group"]] += 1
            stats["estimated_tokens"] += record["estimated_tokens"]
            if record["contains_reasoning"]:
                source_reasoning += 1
                stats["reasoning_examples"] += 1

        if source_written < target_examples:
            warnings.append(f"{source['id']}: materialized {source_written} examples below cap {target_examples}")
        source_summaries.append(
            {
                "id": source["id"],
                "group": source["group"],
                "hf_dataset": source["hf_dataset"],
                "config": config_name or "default",
                "split": split_name,
                "requested_max_examples": target_examples,
                "written_examples": source_written,
                "deduped_examples": source_deduped,
                "invalid_examples": source_invalid,
                "reasoning_examples": source_reasoning,
            }
        )

    with output_jsonl.open("w", encoding="utf-8") as handle:
        for record in records:
            handle.write(json.dumps(record, ensure_ascii=False) + "\n")

    summary = {
        "ok": True,
        "manifest": manifest_summary["manifest"],
        "metadata": str(metadata_path),
        "output_jsonl": str(output_jsonl),
        "record_count": len(records),
        "group_counts": dict(sorted(group_counts.items())),
        "source_counts": dict(sorted(source_counts.items())),
        "stats": {
            "deduped_examples": stats["deduped_examples"],
            "invalid_examples": stats["invalid_examples"],
            "reasoning_examples": stats["reasoning_examples"],
            "estimated_tokens": stats["estimated_tokens"],
        },
        "sources": source_summaries,
        "warnings": warnings,
    }
    output_summary.write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(summary, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
