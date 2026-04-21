#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any
from urllib.parse import quote
from urllib.request import Request, urlopen

from qwen35_workflow_trace_sft_manifest_utils import load_manifest, summarize_manifest


def fetch_json(url: str) -> dict[str, Any]:
    request = Request(url, headers={"User-Agent": "qwen35-sft-metadata-builder/1.0"})
    with urlopen(request, timeout=30) as response:
        return json.loads(response.read().decode("utf-8"))


def first_config_name(payload: dict[str, Any]) -> str:
    configs = payload.get("size", {}).get("configs", [])
    if configs:
        return str(configs[0]["config"])
    return "default"


def first_split_name(payload: dict[str, Any]) -> str:
    splits = payload.get("size", {}).get("splits", [])
    if splits:
        return str(splits[0]["split"])
    return "train"


def infer_schema(features: list[dict[str, Any]]) -> str:
    feature_names = {feature.get("name") for feature in features}
    if "messages" in feature_names:
        return "messages"
    if "conversations" in feature_names:
        return "conversations"
    if {"problem", "thinking", "solution"} <= feature_names:
        return "problem-solution-reasoning"
    prompt_like = {"prompt", "instruction", "input"} & feature_names
    response_like = {"response", "output", "answer", "completion", "chosen"} & feature_names
    if prompt_like and response_like:
        return "prompt-response"
    return "unknown"


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", default="corpora/qwen35-workflow-trace-sft-round-1-manifest.json")
    parser.add_argument("--output", default="output/qwen35-workflow-trace-sft-source-metadata.json")
    args = parser.parse_args()

    manifest_path = Path(args.manifest).resolve()
    output_path = Path(args.output).resolve()
    output_path.parent.mkdir(parents=True, exist_ok=True)

    manifest = load_manifest(manifest_path)
    manifest_summary = summarize_manifest(manifest_path, manifest)
    sources_summary: list[dict[str, Any]] = []
    warnings: list[str] = []

    for source in manifest["sources"]:
        dataset_id = source["hf_dataset"]
        encoded = quote(dataset_id, safe="")
        valid_payload = fetch_json(f"https://datasets-server.huggingface.co/is-valid?dataset={encoded}")
        size_payload = fetch_json(f"https://datasets-server.huggingface.co/size?dataset={encoded}")
        config = first_config_name(size_payload)
        split = first_split_name(size_payload)
        first_rows = fetch_json(
            f"https://datasets-server.huggingface.co/first-rows?dataset={encoded}&config={quote(config, safe='')}&split={quote(split, safe='')}"
        )
        inferred_schema = infer_schema(first_rows.get("features", []))
        if inferred_schema != source["expected_schema"]:
            warnings.append(f"{source['id']}: inferred schema {inferred_schema} != expected {source['expected_schema']}")
        row_count = size_payload.get("size", {}).get("dataset", {}).get("num_rows")
        if isinstance(row_count, int) and row_count < source["max_examples"]:
            warnings.append(f"{source['id']}: num_rows {row_count} < max_examples {source['max_examples']}")
        sources_summary.append(
            {
                "id": source["id"],
                "hf_dataset": dataset_id,
                "sampling_weight": source["sampling_weight"],
                "max_examples": source["max_examples"],
                "validity": valid_payload,
                "dataset_size": size_payload.get("size", {}).get("dataset", {}),
                "selected_config": config,
                "selected_split": split,
                "feature_names": [feature.get("name") for feature in first_rows.get("features", [])],
                "inferred_schema": inferred_schema,
            }
        )

    summary = {
        "ok": True,
        "manifest": manifest_summary["manifest"],
        "source_count": len(sources_summary),
        "sources": sources_summary,
        "warnings": manifest_summary["warnings"] + warnings,
    }
    output_path.write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(summary, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
