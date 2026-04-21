from __future__ import annotations

import json
import random
from collections import Counter
from copy import deepcopy
from pathlib import Path
from typing import Any

import torch
from datasets import Dataset
from transformers import PreTrainedTokenizerBase
from trl.data_utils import maybe_apply_chat_template

from qwen35_workflow_trace_sft_manifest_utils import ensure


def load_json(path: Path) -> dict[str, Any]:
    try:
        return json.loads(path.read_text())
    except FileNotFoundError as exc:
        raise SystemExit(f"Config not found: {path}") from exc
    except json.JSONDecodeError as exc:
        raise SystemExit(f"Config contains invalid JSON: {exc}") from exc


def select_compute_dtype() -> torch.dtype:
    if torch.cuda.is_available() and torch.cuda.is_bf16_supported():
        return torch.bfloat16
    return torch.float16


def has_stable_prompt_prefix(
    tokenizer: PreTrainedTokenizerBase,
    prompt_text: str,
    completion_text: str,
) -> bool:
    prompt_ids = tokenizer(prompt_text, add_special_tokens=False)["input_ids"]
    prompt_completion_ids = tokenizer(prompt_text + completion_text, add_special_tokens=False)["input_ids"]
    return prompt_completion_ids[: len(prompt_ids)] == prompt_ids


def initialize_build_stats() -> dict[str, int]:
    return {
        "rows_seen": 0,
        "rows_kept": 0,
        "sampled_rows": 0,
        "skipped_non_terminal_assistant": 0,
        "skipped_prefix_mismatch": 0,
        "scan_completed": 1,
    }


def close_unbalanced_think_block(content: str) -> str:
    stripped = content.strip()
    lowered = stripped.lower()
    if "<think>" in lowered and "</think>" not in lowered:
        return f"{stripped}\n</think>"
    return content


def repair_training_row(row: dict[str, Any]) -> dict[str, Any]:
    messages = row.get("messages")
    if not isinstance(messages, list) or not messages:
        return row
    last_message = messages[-1]
    if not isinstance(last_message, dict) or last_message.get("role") != "assistant":
        return row
    content = last_message.get("content")
    if not isinstance(content, str):
        return row
    repaired_content = close_unbalanced_think_block(content)
    if repaired_content == content:
        return row
    repaired_row = deepcopy(row)
    repaired_row["messages"][-1]["content"] = repaired_content
    return repaired_row


def convert_training_row(
    row: dict[str, Any],
    tokenizer: PreTrainedTokenizerBase,
) -> tuple[dict[str, Any] | None, str | None]:
    row = repair_training_row(row)
    messages = row["messages"]
    ensure(isinstance(messages, list) and len(messages) >= 2, "Each row must contain at least 2 messages")
    assistant_indices = [index for index, message in enumerate(messages) if message["role"] == "assistant"]
    ensure(assistant_indices, "Each row must contain at least one assistant turn")
    last_assistant_index = assistant_indices[-1]
    if last_assistant_index != len(messages) - 1:
        return None, "skipped_non_terminal_assistant"
    prompt = messages[:last_assistant_index]
    completion = [messages[last_assistant_index]]
    ensure(prompt, "Prompt cannot be empty after splitting the final assistant turn")
    chat_template_kwargs: dict[str, Any] = {}
    if row.get("group") == "chat_quality":
        chat_template_kwargs["enable_thinking"] = False
    rendered = maybe_apply_chat_template(
        {"prompt": prompt, "completion": completion},
        tokenizer,
        **chat_template_kwargs,
    )
    prompt_text = rendered["prompt"]
    completion_text = rendered["completion"]
    if not has_stable_prompt_prefix(tokenizer, prompt_text, completion_text):
        return None, "skipped_prefix_mismatch"
    return {
        "prompt": prompt_text,
        "completion": completion_text,
        "source_id": row["source_id"],
        "contains_reasoning": row["contains_reasoning"],
        "estimated_tokens": row["estimated_tokens"],
    }, None


def build_prompt_completion_dataset(
    jsonl_path: Path,
    tokenizer: PreTrainedTokenizerBase,
    sample_limit: int | None,
    seed: int = 42,
) -> tuple[Dataset, dict[str, int]]:
    ensure(sample_limit is None or sample_limit > 0, "sample_limit must be > 0")
    converted_rows = []
    rng = random.Random(seed)
    stats = initialize_build_stats()
    try:
        with jsonl_path.open(encoding="utf-8") as handle:
            for line in handle:
                if not line.strip():
                    continue
                row = json.loads(line)
                stats["rows_seen"] += 1
                converted_row, skip_reason = convert_training_row(row, tokenizer)
                if skip_reason is not None:
                    stats[skip_reason] += 1
                    continue
                stats["rows_kept"] += 1
                if sample_limit is None:
                    converted_rows.append(converted_row)
                    continue
                if len(converted_rows) < sample_limit:
                    converted_rows.append(converted_row)
                    continue
                replacement_index = rng.randint(0, stats["rows_kept"] - 1)
                if replacement_index < sample_limit:
                    converted_rows[replacement_index] = converted_row
    except FileNotFoundError as exc:
        raise SystemExit(f"Training shard not found: {jsonl_path}") from exc

    dataset = Dataset.from_list(converted_rows).shuffle(seed=seed)
    ensure(len(dataset) > 0, "Converted dataset is empty")
    stats["sampled_rows"] = len(dataset)
    return dataset, stats


def audit_prompt_completion_dataset(
    jsonl_path: Path,
    tokenizer: PreTrainedTokenizerBase,
    sample_examples_per_reason: int = 3,
) -> dict[str, Any]:
    ensure(sample_examples_per_reason > 0, "sample_examples_per_reason must be > 0")
    stats = initialize_build_stats()
    source_stats: dict[str, Counter[str]] = {}
    sample_rows: dict[str, list[dict[str, Any]]] = {
        "skipped_non_terminal_assistant": [],
        "skipped_prefix_mismatch": [],
    }
    try:
        with jsonl_path.open(encoding="utf-8") as handle:
            for line in handle:
                if not line.strip():
                    continue
                row = json.loads(line)
                source_id = str(row.get("source_id", "unknown"))
                stats["rows_seen"] += 1
                per_source = source_stats.setdefault(source_id, Counter())
                per_source["rows_seen"] += 1
                converted_row, skip_reason = convert_training_row(row, tokenizer)
                if skip_reason is not None:
                    stats[skip_reason] += 1
                    per_source[skip_reason] += 1
                    if len(sample_rows[skip_reason]) < sample_examples_per_reason:
                        sample_rows[skip_reason].append({
                            "source_id": source_id,
                            "message_count": len(row.get("messages", [])),
                        })
                    continue
                stats["rows_kept"] += 1
                per_source["rows_kept"] += 1
                per_source["estimated_tokens"] += int(converted_row["estimated_tokens"])
                if converted_row["contains_reasoning"]:
                    per_source["reasoning_rows"] += 1
    except FileNotFoundError as exc:
        raise SystemExit(f"Training shard not found: {jsonl_path}") from exc

    return {
        "stats": stats,
        "source_stats": {
            source_id: dict(sorted(counter.items()))
            for source_id, counter in sorted(source_stats.items())
        },
        "sample_rows": sample_rows,
    }


def summarize_dataset(dataset: Dataset) -> dict[str, Any]:
    source_counts: dict[str, int] = {}
    reasoning_count = 0
    token_estimate = 0
    for row in dataset:
        source_counts[row["source_id"]] = source_counts.get(row["source_id"], 0) + 1
        reasoning_count += 1 if row["contains_reasoning"] else 0
        token_estimate += int(row["estimated_tokens"])
    return {
        "row_count": len(dataset),
        "source_counts": dict(sorted(source_counts.items())),
        "reasoning_rows": reasoning_count,
        "estimated_tokens": token_estimate,
    }
