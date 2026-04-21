#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

from transformers import AutoTokenizer

from qwen35_workflow_trace_sft_train_utils import audit_prompt_completion_dataset, load_json


def resolve_workspace_path(workspace_root: Path, value: str) -> Path:
    path = Path(value)
    if path.is_absolute():
        return path
    return (workspace_root / path).resolve()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", default="configs/qwen35-workflow-trace-sft-round-1.json")
    parser.add_argument("--train-jsonl", default="output/qwen35-workflow-trace-sft-shard.jsonl")
    parser.add_argument("--output", default="output/qwen35-workflow-trace-sft-prefix-audit.json")
    parser.add_argument("--sample-rows-per-reason", type=int, default=3)
    args = parser.parse_args()

    config_path = Path(args.config).resolve()
    train_jsonl_path = Path(args.train_jsonl).resolve()
    output_path = Path(args.output).resolve()
    output_path.parent.mkdir(parents=True, exist_ok=True)
    workspace_root = Path(__file__).resolve().parent.parent

    train_config = load_json(config_path)
    tokenizer_path = resolve_workspace_path(workspace_root, train_config["tokenizer_path"])
    tokenizer = AutoTokenizer.from_pretrained(str(tokenizer_path), trust_remote_code=True)
    if tokenizer.pad_token is None:
        tokenizer.pad_token = tokenizer.eos_token

    audit = audit_prompt_completion_dataset(
        train_jsonl_path,
        tokenizer,
        sample_examples_per_reason=args.sample_rows_per_reason,
    )
    report = {
        "ok": True,
        "config": str(config_path),
        "train_jsonl": str(train_jsonl_path),
        "tokenizer_path": str(tokenizer_path),
        "sample_rows_per_reason": args.sample_rows_per_reason,
        **audit,
    }
    output_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(report, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
