#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from copy import deepcopy
from decimal import Decimal, ROUND_FLOOR
from pathlib import Path
from typing import Any

from qwen35_workflow_trace_sft_manifest_utils import ensure, load_manifest, summarize_manifest


def load_source_estimated_tokens(input_jsonl_path: Path) -> dict[str, int]:
    totals: dict[str, int] = {}
    try:
        with input_jsonl_path.open(encoding="utf-8") as handle:
            for line in handle:
                if not line.strip():
                    continue
                row = json.loads(line)
                source_id = str(row.get("source_id", ""))
                if not source_id:
                    continue
                totals[source_id] = totals.get(source_id, 0) + int(row.get("estimated_tokens", 0))
    except FileNotFoundError as exc:
        raise SystemExit(f"Input shard not found: {input_jsonl_path}") from exc
    return totals


def allocate_with_caps(total: int, weights: dict[str, float], caps: dict[str, int]) -> dict[str, int]:
    ensure(total >= 0, "Allocation total must be >= 0")
    allocations = {item_id: 0 for item_id in weights}
    remaining = total
    active = {item_id for item_id, cap in caps.items() if cap > 0}
    while remaining > 0 and active:
        decimal_weights = {item_id: Decimal(str(weights[item_id])) for item_id in active}
        weight_sum = sum(decimal_weights.values(), Decimal("0"))
        ensure(weight_sum > 0, "Active allocation weights must stay > 0")
        exact_shares = {
            item_id: Decimal(remaining) * decimal_weights[item_id] / weight_sum
            for item_id in active
        }
        base_adds = {
            item_id: min(
                caps[item_id] - allocations[item_id],
                int(exact_shares[item_id].to_integral_value(rounding=ROUND_FLOOR)),
            )
            for item_id in active
        }
        progress = sum(base_adds.values())
        for item_id, base_add in base_adds.items():
            allocations[item_id] += base_add
        remaining -= progress
        remainders: list[tuple[float, float, str]] = []
        for item_id in active:
            free_capacity = caps[item_id] - allocations[item_id]
            if free_capacity <= 0:
                continue
            floored = exact_shares[item_id].to_integral_value(rounding=ROUND_FLOOR)
            remainders.append((exact_shares[item_id] - floored, weights[item_id], item_id))
        if remaining == 0:
            break
        active = {item_id for item_id in active if allocations[item_id] < caps[item_id]}
        if not active:
            break
        if progress == 0:
            for _fractional, _weight, item_id in sorted(remainders, reverse=True):
                if remaining == 0:
                    break
                if allocations[item_id] >= caps[item_id]:
                    continue
                allocations[item_id] += 1
                remaining -= 1
            active = {item_id for item_id in active if allocations[item_id] < caps[item_id]}
    return allocations


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--base-manifest", default="corpora/qwen35-workflow-trace-sft-round-1-manifest.json")
    parser.add_argument("--audit", default="output/qwen35-workflow-trace-sft-prefix-audit.json")
    parser.add_argument("--recovery-probe", default="")
    parser.add_argument("--input-jsonl", default="output/qwen35-workflow-trace-sft-shard.jsonl")
    parser.add_argument("--output-manifest", default="corpora/qwen35-workflow-trace-sft-round-1-search-ready-manifest.json")
    parser.add_argument("--output-summary", default="output/qwen35-workflow-trace-sft-search-ready-manifest-summary.json")
    parser.add_argument("--min-clean-examples", type=int, default=1)
    args = parser.parse_args()

    base_manifest_path = Path(args.base_manifest).resolve()
    audit_path = Path(args.audit).resolve()
    recovery_probe_path = Path(args.recovery_probe).resolve() if args.recovery_probe else None
    input_jsonl_path = Path(args.input_jsonl).resolve()
    output_manifest_path = Path(args.output_manifest).resolve()
    output_summary_path = Path(args.output_summary).resolve()
    output_manifest_path.parent.mkdir(parents=True, exist_ok=True)
    output_summary_path.parent.mkdir(parents=True, exist_ok=True)

    base_manifest = load_manifest(base_manifest_path)
    base_manifest_summary = summarize_manifest(base_manifest_path, base_manifest)
    try:
        audit = json.loads(audit_path.read_text())
    except FileNotFoundError as exc:
        raise SystemExit(f"Audit not found: {audit_path}") from exc
    except json.JSONDecodeError as exc:
        raise SystemExit(f"Audit contains invalid JSON: {exc}") from exc

    recovery_probe = None
    if recovery_probe_path is not None:
        try:
            recovery_probe = json.loads(recovery_probe_path.read_text())
        except FileNotFoundError as exc:
            raise SystemExit(f"Recovery probe not found: {recovery_probe_path}") from exc
        except json.JSONDecodeError as exc:
            raise SystemExit(f"Recovery probe contains invalid JSON: {exc}") from exc

    audit_sources = audit.get("source_stats", {})
    source_estimated_tokens_from_shard = load_source_estimated_tokens(input_jsonl_path)
    recovery_sources = {}
    if recovery_probe is not None:
        recovery_sources = {
            source["source_id"]: source
            for source in recovery_probe.get("sources", [])
            if isinstance(source, dict) and isinstance(source.get("source_id"), str)
        }
    group_weight_index = {group["id"]: float(group["target_weight"]) for group in base_manifest["mixture_groups"]}
    clean_supply_by_source: dict[str, int] = {}
    clean_supply_by_group: dict[str, int] = {}
    active_groups: list[str] = []
    active_sources: list[dict[str, Any]] = []
    dropped_sources: list[dict[str, Any]] = []

    for source in base_manifest["sources"]:
        source_stats = audit_sources.get(source["id"], {})
        clean_supply = int(source_stats.get("rows_kept", 0))
        recovery_summary = recovery_sources.get(source["id"])
        if recovery_summary is not None:
            current_safe = int(recovery_summary.get("current_prefix_safe_rows", clean_supply))
            recovery_candidates = []
            for strategy_summary in recovery_summary.get("recovery_strategies", {}).values():
                if not isinstance(strategy_summary, dict):
                    continue
                recovery_candidates.append(int(strategy_summary.get("prefix_safe_rows", 0)))
            recovered_clean_supply = min(
                int(recovery_summary.get("rows_seen", current_safe)),
                current_safe + max(recovery_candidates, default=0),
            )
            clean_supply = max(clean_supply, recovered_clean_supply)
        clean_supply_by_source[source["id"]] = clean_supply
        clean_supply_by_group[source["group"]] = clean_supply_by_group.get(source["group"], 0) + clean_supply
        source_copy = deepcopy(source)
        source_copy["clean_examples_available"] = clean_supply
        source_copy["clean_recovery_rate"] = round(clean_supply / source["max_examples"], 6)
        if clean_supply >= args.min_clean_examples:
            active_sources.append(source_copy)
        else:
            dropped_sources.append(source_copy)

    for group in base_manifest["mixture_groups"]:
        if clean_supply_by_group.get(group["id"], 0) >= args.min_clean_examples:
            active_groups.append(group["id"])

    ensure(active_groups, "No active mixture groups survived the audit")
    active_group_weight_sum = sum(group_weight_index[group_id] for group_id in active_groups)
    normalized_group_weights = {
        group_id: Decimal(str(group_weight_index[group_id])) / Decimal(str(active_group_weight_sum))
        for group_id in active_groups
    }
    max_balanced_examples = min(
        int(
            (
                Decimal(clean_supply_by_group[group_id]) / normalized_group_weights[group_id]
            ).to_integral_value(rounding=ROUND_FLOOR)
        )
        for group_id in active_groups
    )
    ensure(max_balanced_examples > 0, "Balanced target_examples resolved to 0 after audit")

    group_caps = {group_id: clean_supply_by_group[group_id] for group_id in active_groups}
    group_allocations = allocate_with_caps(max_balanced_examples, normalized_group_weights, group_caps)
    target_examples = sum(group_allocations.values())
    ensure(target_examples > 0, "No examples allocated for search-ready manifest")

    output_sources: list[dict[str, Any]] = []
    estimated_target_token_budget = 0
    per_group_source_allocations: dict[str, dict[str, int]] = {}
    for group_id in active_groups:
        group_sources = [source for source in active_sources if source["group"] == group_id]
        group_target = group_allocations[group_id]
        group_weight_total = sum(Decimal(str(source["sampling_weight"])) for source in group_sources)
        source_weights = {
            source["id"]: float(Decimal(str(source["sampling_weight"])) / group_weight_total)
            for source in group_sources
        }
        source_caps = {source["id"]: int(source["clean_examples_available"]) for source in group_sources}
        allocations = allocate_with_caps(group_target, source_weights, source_caps)
        per_group_source_allocations[group_id] = allocations
        for source in group_sources:
            allocated = allocations[source["id"]]
            if allocated <= 0:
                dropped_sources.append(source)
                continue
            source["max_examples"] = allocated
            source["sampling_weight"] = allocated / target_examples
            source["search_ready_clean_examples_available"] = source["clean_examples_available"]
            source_estimated_tokens = source_estimated_tokens_from_shard.get(source["id"], 0)
            clean_examples_available = max(1, int(source["clean_examples_available"]))
            source["estimated_tokens_per_example"] = round(source_estimated_tokens / clean_examples_available, 2)
            estimated_target_token_budget += round((source_estimated_tokens / clean_examples_available) * allocated)
            output_sources.append(source)

    output_groups = []
    for group in base_manifest["mixture_groups"]:
        group_id = group["id"]
        if group_id not in group_allocations or group_allocations[group_id] <= 0:
            continue
        group_copy = deepcopy(group)
        group_copy["target_weight"] = group_allocations[group_id] / target_examples
        group_copy["clean_examples_available"] = clean_supply_by_group[group_id]
        output_groups.append(group_copy)

    output_manifest = deepcopy(base_manifest)
    output_manifest["manifest_stage"] = "search-ready"
    output_manifest["description"] = (
        f"{base_manifest['description']} Search-ready rebalance after full-shard prefix audit."
    )
    output_manifest["round_id"] = f"{base_manifest['round_id']}-search-ready"
    output_manifest["target_examples"] = target_examples
    output_manifest["target_token_budget"] = estimated_target_token_budget
    output_manifest["mixture_groups"] = output_groups
    output_manifest["sources"] = output_sources
    output_manifest["derived_from_manifest"] = str(base_manifest_path)
    output_manifest["derived_from_audit"] = str(audit_path)

    summarize_manifest(output_manifest_path, output_manifest)
    output_manifest_path.write_text(json.dumps(output_manifest, ensure_ascii=False, indent=2) + "\n")

    summary = {
        "ok": True,
        "base_manifest": base_manifest_summary["manifest"],
        "audit": str(audit_path),
        "recovery_probe": str(recovery_probe_path) if recovery_probe_path is not None else "",
        "input_jsonl": str(input_jsonl_path),
        "output_manifest": str(output_manifest_path),
        "output_target_examples": target_examples,
        "output_target_token_budget": estimated_target_token_budget,
        "max_balanced_examples_before_caps": max_balanced_examples,
        "active_groups": {
            group_id: {
                "original_target_weight": round(float(group_weight_index[group_id]), 6),
                "normalized_active_weight": round(float(normalized_group_weights[group_id]), 6),
                "clean_examples_available": clean_supply_by_group[group_id],
                "allocated_examples": group_allocations[group_id],
            }
            for group_id in active_groups
        },
        "active_sources": [
            {
                "id": source["id"],
                "group": source["group"],
                "clean_examples_available": source["clean_examples_available"],
                "clean_recovery_rate": source["clean_recovery_rate"],
                "allocated_examples": source["max_examples"],
                "allocated_sampling_weight": round(source["sampling_weight"], 6),
                "estimated_tokens_per_example": source["estimated_tokens_per_example"],
            }
            for source in output_sources
        ],
        "dropped_sources": [
            {
                "id": source["id"],
                "group": source["group"],
                "clean_examples_available": source["clean_examples_available"],
                "clean_recovery_rate": source["clean_recovery_rate"],
            }
            for source in dropped_sources
            if source["id"] not in {output_source["id"] for output_source in output_sources}
        ],
        "warnings": [
            "chat_quality dropped from the search-ready manifest because the current audit exposes zero clean rows.",
        ] if clean_supply_by_group.get("chat_quality", 0) == 0 else [],
    }
    output_summary_path.write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(summary, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
