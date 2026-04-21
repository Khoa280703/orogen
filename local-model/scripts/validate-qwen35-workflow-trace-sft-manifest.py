#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

from qwen35_workflow_trace_sft_manifest_utils import load_manifest, summarize_manifest


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", default="corpora/qwen35-workflow-trace-sft-round-1-manifest.json")
    parser.add_argument("--output", default="")
    args = parser.parse_args()

    manifest_path = Path(args.manifest).resolve()
    manifest = load_manifest(manifest_path)
    summary = summarize_manifest(manifest_path, manifest)
    output = json.dumps(summary, ensure_ascii=False, indent=2) + "\n"
    if args.output:
        output_path = Path(args.output).resolve()
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(output)
    print(output, end="")


if __name__ == "__main__":
    main()
