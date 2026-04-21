from __future__ import annotations

import json
from collections import Counter
from pathlib import Path
from typing import Any


def ensure(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def load_manifest(path: Path) -> dict[str, Any]:
    try:
        payload = json.loads(path.read_text())
    except FileNotFoundError as exc:
        raise SystemExit(f"Manifest not found: {path}") from exc
    except json.JSONDecodeError as exc:
        raise SystemExit(f"Manifest contains invalid JSON: {exc}") from exc
    ensure(isinstance(payload, dict), "Manifest must be a JSON object.")
    return payload


def summarize_manifest(manifest_path: Path, manifest: dict[str, Any]) -> dict[str, Any]:
    ensure(manifest.get("format") == "qwen35-workflow-trace-sft-manifest-v1", "Unsupported manifest format.")
    ensure(isinstance(manifest.get("model"), str) and manifest["model"], "Manifest is missing model.")
    ensure(isinstance(manifest.get("round_id"), str) and manifest["round_id"], "Manifest is missing round_id.")
    ensure(isinstance(manifest.get("language"), str) and manifest["language"], "Manifest is missing language.")
    ensure(isinstance(manifest.get("target_examples"), int) and manifest["target_examples"] > 0, "Invalid target_examples.")
    ensure(isinstance(manifest.get("target_token_budget"), int) and manifest["target_token_budget"] > 0, "Invalid target_token_budget.")
    ensure(isinstance(manifest.get("max_sequence_length"), int) and manifest["max_sequence_length"] > 0, "Invalid max_sequence_length.")
    ensure(isinstance(manifest.get("response_only"), bool), "response_only must be boolean.")
    ensure(isinstance(manifest.get("mixture_groups"), list) and manifest["mixture_groups"], "Manifest must contain mixture_groups.")
    ensure(isinstance(manifest.get("sources"), list) and manifest["sources"], "Manifest must contain sources.")

    group_weights: dict[str, float] = {}
    for group in manifest["mixture_groups"]:
        ensure(isinstance(group, dict), "Each mixture group must be an object.")
        group_id = group.get("id")
        ensure(isinstance(group_id, str) and group_id, "Each mixture group needs an id.")
        target_weight = group.get("target_weight")
        ensure(isinstance(target_weight, (int, float)) and target_weight > 0, f"{group_id}: invalid target_weight")
        group_weights[group_id] = float(target_weight)
    ensure(abs(sum(group_weights.values()) - 1.0) <= 1e-6, "mixture_groups target_weight must sum to 1.0")

    source_ids: set[str] = set()
    group_counts: Counter[str] = Counter()
    commercial_counts: Counter[str] = Counter()
    schema_counts: Counter[str] = Counter()
    group_weight_totals: Counter[str] = Counter()
    max_example_total = 0

    for source in manifest["sources"]:
        ensure(isinstance(source, dict), "Each source must be an object.")
        source_id = source.get("id")
        ensure(isinstance(source_id, str) and source_id, "Each source needs an id.")
        ensure(source_id not in source_ids, f"Duplicate source id: {source_id}")
        source_ids.add(source_id)
        ensure(source.get("kind") == "hf-dataset", f"{source_id}: only hf-dataset is supported in this manifest")
        group_id = source.get("group")
        ensure(group_id in group_weights, f"{source_id}: unknown group {group_id}")
        hf_dataset = source.get("hf_dataset")
        ensure(isinstance(hf_dataset, str) and "/" in hf_dataset, f"{source_id}: invalid hf_dataset")
        sampling_weight = source.get("sampling_weight")
        ensure(isinstance(sampling_weight, (int, float)) and sampling_weight > 0, f"{source_id}: invalid sampling_weight")
        max_examples = source.get("max_examples")
        ensure(isinstance(max_examples, int) and max_examples > 0, f"{source_id}: invalid max_examples")
        preferred_configs = source.get("preferred_configs")
        preferred_splits = source.get("preferred_splits")
        ensure(isinstance(preferred_configs, list) and preferred_configs, f"{source_id}: preferred_configs required")
        ensure(isinstance(preferred_splits, list) and preferred_splits, f"{source_id}: preferred_splits required")
        normalization = source.get("normalization")
        ensure(isinstance(normalization, dict) and isinstance(normalization.get("strategy"), str), f"{source_id}: normalization.strategy required")
        expected_schema = source.get("expected_schema")
        ensure(isinstance(expected_schema, str) and expected_schema, f"{source_id}: expected_schema required")
        group_counts[group_id] += 1
        commercial_counts[source.get("commercial_status", "unknown")] += 1
        schema_counts[expected_schema] += 1
        group_weight_totals[group_id] += float(sampling_weight)
        max_example_total += max_examples

    ensure(abs(sum(group_weight_totals.values()) - 1.0) <= 1e-6, "sources sampling_weight must sum to 1.0")
    for group_id, target_weight in sorted(group_weights.items()):
        actual = float(group_weight_totals[group_id])
        ensure(abs(actual - target_weight) <= 1e-6, f"{group_id}: source sampling_weight total {actual:.6f} != target_weight {target_weight:.6f}")

    warnings: list[str] = []
    if commercial_counts.get("review-required", 0):
        warnings.append("Some SFT sources require legal/licensing review before commercial use.")
    if max_example_total < manifest["target_examples"]:
        warnings.append("Sum of max_examples is lower than target_examples; either raise caps or lower the round target.")

    return {
        "ok": True,
        "manifest": {
            "path": str(manifest_path),
            "format": manifest["format"],
            "model": manifest["model"],
            "round_id": manifest["round_id"],
            "target_examples": manifest["target_examples"],
            "target_token_budget": manifest["target_token_budget"],
            "max_sequence_length": manifest["max_sequence_length"],
            "response_only": manifest["response_only"],
        },
        "summary": {
            "source_count": len(manifest["sources"]),
            "group_counts": dict(sorted(group_counts.items())),
            "group_weight_totals": {key: round(value, 6) for key, value in sorted(group_weight_totals.items())},
            "schema_counts": dict(sorted(schema_counts.items())),
            "commercial_counts": dict(sorted(commercial_counts.items())),
            "max_examples_total": max_example_total,
        },
        "warnings": warnings,
    }
