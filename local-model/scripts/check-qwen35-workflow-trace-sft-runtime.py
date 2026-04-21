#!/usr/bin/env python3
from __future__ import annotations

import argparse
import importlib
import json
import sys
from pathlib import Path


REQUIRED_MODULES = ["torch", "datasets", "transformers", "peft", "trl", "accelerate", "bitsandbytes"]


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", default="output/qwen35-workflow-trace-sft-runtime-check.json")
    args = parser.parse_args()

    output_path = Path(args.output).resolve()
    output_path.parent.mkdir(parents=True, exist_ok=True)

    modules = []
    missing = []
    for name in REQUIRED_MODULES:
        try:
            module = importlib.import_module(name)
            modules.append({"name": name, "available": True, "version": getattr(module, "__version__", "n/a")})
        except Exception as exc:  # noqa: BLE001
            modules.append({"name": name, "available": False, "error": type(exc).__name__})
            missing.append(name)

    summary = {
        "ok": not missing,
        "python_executable": sys.executable,
        "required_modules": modules,
        "missing_modules": missing,
        "recommended_install": (
            "./scripts/setup-qwen35-workflow-trace-sft-runtime.sh"
            if missing else ""
        ),
    }
    output_path.write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(summary, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
