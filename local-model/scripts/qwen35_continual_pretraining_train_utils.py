from __future__ import annotations

import json
import random
from collections import Counter
from pathlib import Path
from typing import Any

from datasets import Dataset
from transformers import PreTrainedTokenizerBase

from qwen35_workflow_trace_sft_manifest_utils import ensure


def initialize_build_stats() -> dict[str, int]:
    return {
        "docs_seen": 0,
        "docs_sampled": 0,
        "docs_kept": 0,
        "short_docs_skipped": 0,
        "empty_docs_skipped": 0,
        "token_blocks_written": 0,
        "remainder_tokens_dropped": 0,
        "remainder_block_written": 0,
    }


def normalize_document_content(document: dict[str, Any]) -> str:
    content = document.get("content")
    ensure(isinstance(content, str), "Each CPT document must contain string content")
    return content.strip()


def reservoir_sample_documents(
    jsonl_path: Path,
    sample_limit: int | None,
    seed: int,
) -> tuple[list[dict[str, Any]], dict[str, int]]:
    rng = random.Random(seed)
    documents: list[dict[str, Any]] = []
    stats = initialize_build_stats()
    try:
        with jsonl_path.open(encoding="utf-8") as handle:
            for line in handle:
                if not line.strip():
                    continue
                document = json.loads(line)
                stats["docs_seen"] += 1
                if sample_limit is None or len(documents) < sample_limit:
                    documents.append(document)
                    continue
                replacement_index = rng.randint(0, stats["docs_seen"] - 1)
                if replacement_index < sample_limit:
                    documents[replacement_index] = document
    except FileNotFoundError as exc:
        raise SystemExit(f"CPT shard not found: {jsonl_path}") from exc
    stats["docs_sampled"] = len(documents)
    return documents, stats


def build_continual_pretraining_dataset(
    jsonl_path: Path,
    tokenizer: PreTrainedTokenizerBase,
    max_sequence_length: int,
    sample_limit: int | None,
    min_doc_chars: int = 40,
    seed: int = 42,
    min_remainder_tokens: int | None = None,
) -> tuple[Dataset, dict[str, Any]]:
    ensure(max_sequence_length > 0, "max_sequence_length must be > 0")
    ensure(sample_limit is None or sample_limit > 0, "sample_limit must be > 0 when provided")
    documents, stats = reservoir_sample_documents(jsonl_path, sample_limit, seed)
    source_counts: Counter[str] = Counter()
    domain_counts: Counter[str] = Counter()
    token_stream: list[int] = []
    eos_token_id = tokenizer.eos_token_id
    ensure(eos_token_id is not None, "Tokenizer must expose eos_token_id")

    for document in documents:
        content = normalize_document_content(document)
        if not content:
            stats["empty_docs_skipped"] += 1
            continue
        if len(content) < min_doc_chars:
            stats["short_docs_skipped"] += 1
            continue
        token_ids = tokenizer(content, add_special_tokens=False)["input_ids"]
        if not token_ids:
            stats["empty_docs_skipped"] += 1
            continue
        token_stream.extend(token_ids)
        token_stream.append(eos_token_id)
        stats["docs_kept"] += 1
        source_counts[str(document.get("source_id", "unknown"))] += 1
        domain_counts[str(document.get("domain", "unknown"))] += 1

    min_remainder = min_remainder_tokens or max(128, max_sequence_length // 4)
    rows: list[dict[str, Any]] = []
    for start in range(0, len(token_stream), max_sequence_length):
        chunk = token_stream[start:start + max_sequence_length]
        if len(chunk) < min_remainder:
            stats["remainder_tokens_dropped"] += len(chunk)
            continue
        if len(chunk) < max_sequence_length:
            stats["remainder_block_written"] += 1
        rows.append(
            {
                "input_ids": chunk,
                "attention_mask": [1] * len(chunk),
            }
        )

    dataset = Dataset.from_list(rows).shuffle(seed=seed)
    ensure(len(dataset) > 0, "Converted CPT dataset is empty")
    stats["token_blocks_written"] = len(dataset)
    return dataset, {
        "stats": stats,
        "source_counts": dict(sorted(source_counts.items())),
        "domain_counts": dict(sorted(domain_counts.items())),
        "token_count": len(token_stream),
        "max_sequence_length": max_sequence_length,
        "min_remainder_tokens": min_remainder,
    }


def summarize_continual_pretraining_dataset(dataset: Dataset) -> dict[str, Any]:
    lengths = [len(row["input_ids"]) for row in dataset]
    total_tokens = sum(lengths)
    return {
        "row_count": len(dataset),
        "total_tokens": total_tokens,
        "min_block_tokens": min(lengths),
        "max_block_tokens": max(lengths),
        "avg_block_tokens": round(total_tokens / len(lengths), 2),
    }


def split_continual_pretraining_dataset(
    dataset: Dataset,
    holdout_ratio: float,
    min_eval_blocks: int,
    seed: int,
) -> tuple[Dataset, Dataset, dict[str, Any]]:
    ensure(0.0 < holdout_ratio < 0.5, "holdout_ratio must be between 0 and 0.5")
    ensure(min_eval_blocks > 0, "min_eval_blocks must be > 0")
    ensure(len(dataset) >= 2, "Need at least 2 token blocks to create a CPT holdout split")

    computed_eval_blocks = max(int(round(len(dataset) * holdout_ratio)), min_eval_blocks)
    eval_blocks = min(max(1, computed_eval_blocks), len(dataset) - 1)
    split = dataset.train_test_split(test_size=eval_blocks, shuffle=True, seed=seed)
    train_dataset = split["train"]
    eval_dataset = split["test"]
    return train_dataset, eval_dataset, {
        "holdout_ratio": holdout_ratio,
        "requested_min_eval_blocks": min_eval_blocks,
        "train_block_count": len(train_dataset),
        "holdout_block_count": len(eval_dataset),
    }
