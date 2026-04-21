#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any

from transformers import AutoTokenizer

from qwen35_workflow_trace_sft_manifest_utils import ensure, load_manifest
from qwen35_workflow_trace_sft_train_utils import convert_training_row, load_json


def resolve_workspace_path(workspace_root: Path, value: str) -> Path:
    path = Path(value)
    if path.is_absolute():
        return path
    return (workspace_root / path).resolve()


def make_empty_think_wrapper(content: str) -> str:
    return f"<think>\n\n</think>\n\n{content}"


def append_missing_close_tag(content: str) -> str:
    return f"{content.rstrip()}\n</think>"


def build_row_variant(row: dict[str, Any], new_content: str) -> dict[str, Any]:
    mutated = json.loads(json.dumps(row, ensure_ascii=False))
    mutated["messages"][-1]["content"] = new_content
    return mutated


def classify_assistant_shape(content: str) -> str:
    stripped = content.strip()
    has_open = "<think>" in stripped.lower()
    has_close = "</think>" in stripped.lower()
    starts_with_open = stripped.lower().startswith("<think>")
    if starts_with_open and has_close:
        return "starts_with_balanced_think"
    if starts_with_open and not has_close:
        return "starts_with_open_think_only"
    if has_open and has_close:
        return "contains_balanced_think"
    if has_open and not has_close:
        return "contains_open_think_only"
    return "plain_answer"


def initialize_source_summary(source: dict[str, Any]) -> dict[str, Any]:
    return {
        "source_id": source["id"],
        "group": source["group"],
        "hf_dataset": source["hf_dataset"],
        "requested_max_examples": source["max_examples"],
        "rows_seen": 0,
        "current_prefix_safe_rows": 0,
        "prefix_fail_rows": 0,
        "shape_counts": {},
        "recovery_strategies": {
            "empty_think_wrapper": {
                "eligible_rows": 0,
                "prefix_safe_rows": 0,
            },
            "append_missing_close_tag": {
                "eligible_rows": 0,
                "prefix_safe_rows": 0,
            },
        },
        "sample_rows": defaultdict(list),
    }


def append_sample(summary: dict[str, Any], bucket: str, row: dict[str, Any], limit: int) -> None:
    samples = summary["sample_rows"][bucket]
    if len(samples) >= limit:
        return
    samples.append(
        {
            "row_index": row.get("row_index"),
            "message_count": len(row.get("messages", [])),
            "assistant_preview": row["messages"][-1]["content"][:240],
        }
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", default="configs/qwen35-workflow-trace-sft-round-1.json")
    parser.add_argument("--manifest", default="corpora/qwen35-workflow-trace-sft-round-1-manifest.json")
    parser.add_argument("--input-jsonl", default="output/qwen35-workflow-trace-sft-shard.jsonl")
    parser.add_argument("--output", default="output/qwen35-workflow-trace-sft-source-recovery-probe.json")
    parser.add_argument("--source-id", action="append", dest="source_ids")
    parser.add_argument("--sample-limit-per-source", type=int, default=3)
    args = parser.parse_args()

    config_path = Path(args.config).resolve()
    manifest_path = Path(args.manifest).resolve()
    input_jsonl = Path(args.input_jsonl).resolve()
    output_path = Path(args.output).resolve()
    output_path.parent.mkdir(parents=True, exist_ok=True)
    workspace_root = Path(__file__).resolve().parent.parent

    train_config = load_json(config_path)
    manifest = load_manifest(manifest_path)
    source_index = {source["id"]: source for source in manifest["sources"]}

    requested_source_ids = args.source_ids or ["chat-quality-guardrail", "open-thoughts-breadth"]
    for source_id in requested_source_ids:
        ensure(source_id in source_index, f"Unknown source_id: {source_id}")

    tokenizer_path = resolve_workspace_path(workspace_root, train_config["tokenizer_path"])
    tokenizer = AutoTokenizer.from_pretrained(str(tokenizer_path), trust_remote_code=True)
    if tokenizer.pad_token is None:
        tokenizer.pad_token = tokenizer.eos_token

    summaries = {source_id: initialize_source_summary(source_index[source_id]) for source_id in requested_source_ids}
    global_stats: Counter[str] = Counter()

    try:
        with input_jsonl.open(encoding="utf-8") as handle:
            for line in handle:
                if not line.strip():
                    continue
                row = json.loads(line)
                source_id = row.get("source_id")
                if source_id not in summaries:
                    continue
                summary = summaries[source_id]
                summary["rows_seen"] += 1
                global_stats["rows_seen"] += 1
                assistant_content = str(row["messages"][-1]["content"])
                shape = classify_assistant_shape(assistant_content)
                shape_counts = Counter(summary["shape_counts"])
                shape_counts[shape] += 1
                summary["shape_counts"] = dict(sorted(shape_counts.items()))

                converted_row, skip_reason = convert_training_row(row, tokenizer)
                if skip_reason is None:
                    summary["current_prefix_safe_rows"] += 1
                    global_stats["current_prefix_safe_rows"] += 1
                else:
                    summary["prefix_fail_rows"] += 1
                    global_stats[skip_reason] += 1
                    append_sample(summary, skip_reason, row, args.sample_limit_per_source)

                if shape == "plain_answer":
                    summary["recovery_strategies"]["empty_think_wrapper"]["eligible_rows"] += 1
                    alt_row = build_row_variant(row, make_empty_think_wrapper(assistant_content))
                    _, alt_skip_reason = convert_training_row(alt_row, tokenizer)
                    if alt_skip_reason is None:
                        summary["recovery_strategies"]["empty_think_wrapper"]["prefix_safe_rows"] += 1
                        append_sample(summary, "empty_think_wrapper", alt_row, args.sample_limit_per_source)

                if shape in {"starts_with_open_think_only", "contains_open_think_only"}:
                    summary["recovery_strategies"]["append_missing_close_tag"]["eligible_rows"] += 1
                    alt_row = build_row_variant(row, append_missing_close_tag(assistant_content))
                    _, alt_skip_reason = convert_training_row(alt_row, tokenizer)
                    if alt_skip_reason is None:
                        summary["recovery_strategies"]["append_missing_close_tag"]["prefix_safe_rows"] += 1
                        append_sample(summary, "append_missing_close_tag", alt_row, args.sample_limit_per_source)
    except FileNotFoundError as exc:
        raise SystemExit(f"Input shard not found: {input_jsonl}") from exc

    report_sources: list[dict[str, Any]] = []
    for source_id in requested_source_ids:
        summary = summaries[source_id]
        rows_seen = summary["rows_seen"]
        current_safe = summary["current_prefix_safe_rows"]
        summary["current_prefix_safe_rate"] = round(current_safe / rows_seen, 6) if rows_seen else 0.0
        for strategy_summary in summary["recovery_strategies"].values():
            eligible = strategy_summary["eligible_rows"]
            safe_rows = strategy_summary["prefix_safe_rows"]
            strategy_summary["prefix_safe_rate"] = round(safe_rows / eligible, 6) if eligible else 0.0
        summary["sample_rows"] = {
            bucket: rows for bucket, rows in sorted(summary["sample_rows"].items())
        }
        report_sources.append(summary)

    report = {
        "ok": True,
        "config": str(config_path),
        "manifest": str(manifest_path),
        "input_jsonl": str(input_jsonl),
        "tokenizer_path": str(tokenizer_path),
        "requested_source_ids": requested_source_ids,
        "global_stats": dict(sorted(global_stats.items())),
        "sources": report_sources,
    }
    output_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(report, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
