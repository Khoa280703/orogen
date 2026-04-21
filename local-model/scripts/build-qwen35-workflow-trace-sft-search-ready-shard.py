#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from collections import Counter
from pathlib import Path
from typing import Any

from transformers import AutoTokenizer

from qwen35_workflow_trace_sft_manifest_utils import ensure, load_manifest, summarize_manifest
from qwen35_workflow_trace_sft_train_utils import convert_training_row, load_json, repair_training_row


def resolve_workspace_path(workspace_root: Path, value: str) -> Path:
    path = Path(value)
    if path.is_absolute():
        return path
    return (workspace_root / path).resolve()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", default="configs/qwen35-workflow-trace-sft-round-1.json")
    parser.add_argument("--manifest", default="corpora/qwen35-workflow-trace-sft-round-1-search-ready-manifest.json")
    parser.add_argument("--input-jsonl", default="output/qwen35-workflow-trace-sft-shard.jsonl")
    parser.add_argument("--output-jsonl", default="output/qwen35-workflow-trace-sft-search-ready-shard.jsonl")
    parser.add_argument("--output-summary", default="output/qwen35-workflow-trace-sft-search-ready-shard-summary.json")
    args = parser.parse_args()

    config_path = Path(args.config).resolve()
    manifest_path = Path(args.manifest).resolve()
    input_jsonl = Path(args.input_jsonl).resolve()
    output_jsonl = Path(args.output_jsonl).resolve()
    output_summary = Path(args.output_summary).resolve()
    output_jsonl.parent.mkdir(parents=True, exist_ok=True)
    output_summary.parent.mkdir(parents=True, exist_ok=True)
    workspace_root = Path(__file__).resolve().parent.parent

    train_config = load_json(config_path)
    manifest = load_manifest(manifest_path)
    manifest_summary = summarize_manifest(manifest_path, manifest)
    tokenizer_path = resolve_workspace_path(workspace_root, train_config["tokenizer_path"])
    tokenizer = AutoTokenizer.from_pretrained(str(tokenizer_path), trust_remote_code=True)
    if tokenizer.pad_token is None:
        tokenizer.pad_token = tokenizer.eos_token

    source_caps = {source["id"]: int(source["max_examples"]) for source in manifest["sources"]}
    source_groups = {source["id"]: source["group"] for source in manifest["sources"]}
    source_requested = {source["id"]: int(source["max_examples"]) for source in manifest["sources"]}
    source_counts: Counter[str] = Counter()
    group_counts: Counter[str] = Counter()
    stats: Counter[str] = Counter()
    source_summaries = {
        source["id"]: {
            "id": source["id"],
            "group": source["group"],
            "requested_examples": int(source["max_examples"]),
            "written_examples": 0,
            "skipped_prefix_mismatch": 0,
            "skipped_non_terminal_assistant": 0,
        }
        for source in manifest["sources"]
    }

    try:
        with input_jsonl.open(encoding="utf-8") as reader, output_jsonl.open("w", encoding="utf-8") as writer:
            for line in reader:
                if not line.strip():
                    continue
                row = json.loads(line)
                source_id = row.get("source_id")
                stats["rows_seen"] += 1
                if source_id not in source_caps:
                    stats["skipped_inactive_source"] += 1
                    continue
                if source_counts[source_id] >= source_caps[source_id]:
                    stats["skipped_over_cap"] += 1
                    continue
                repaired_row = repair_training_row(row)
                converted_row, skip_reason = convert_training_row(repaired_row, tokenizer)
                if skip_reason is not None:
                    stats[skip_reason] += 1
                    source_summaries[source_id][skip_reason] += 1
                    continue
                writer.write(json.dumps(repaired_row, ensure_ascii=False) + "\n")
                source_counts[source_id] += 1
                group_counts[source_groups[source_id]] += 1
                source_summaries[source_id]["written_examples"] += 1
                stats["rows_written"] += 1
                stats["reasoning_examples"] += 1 if row.get("contains_reasoning") else 0
                stats["estimated_tokens"] += int(row.get("estimated_tokens", 0))
    except FileNotFoundError as exc:
        raise SystemExit(f"Input shard not found: {input_jsonl}") from exc

    warnings = []
    for source_id, requested in source_requested.items():
        written = source_counts[source_id]
        if written < requested:
            warnings.append(f"{source_id}: wrote {written} examples below requested {requested}")

    summary = {
        "ok": not warnings,
        "config": str(config_path),
        "manifest": manifest_summary["manifest"],
        "input_jsonl": str(input_jsonl),
        "output_jsonl": str(output_jsonl),
        "tokenizer_path": str(tokenizer_path),
        "record_count": stats["rows_written"],
        "group_counts": dict(sorted(group_counts.items())),
        "source_counts": dict(sorted(source_counts.items())),
        "stats": {
            "rows_seen": stats["rows_seen"],
            "rows_written": stats["rows_written"],
            "skipped_inactive_source": stats["skipped_inactive_source"],
            "skipped_over_cap": stats["skipped_over_cap"],
            "skipped_non_terminal_assistant": stats["skipped_non_terminal_assistant"],
            "skipped_prefix_mismatch": stats["skipped_prefix_mismatch"],
            "reasoning_examples": stats["reasoning_examples"],
            "estimated_tokens": stats["estimated_tokens"],
        },
        "sources": [source_summaries[source["id"]] for source in manifest["sources"]],
        "warnings": warnings,
    }
    output_summary.write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(summary, ensure_ascii=False, indent=2))
    if warnings:
        raise SystemExit("Search-ready shard underfilled; see summary output.")


if __name__ == "__main__":
    main()
