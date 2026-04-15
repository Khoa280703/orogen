#!/usr/bin/env python3
from __future__ import annotations

import argparse
import importlib.util
import json
from datetime import datetime, timezone
from pathlib import Path
from urllib.error import HTTPError, URLError


def load_support_module() -> object:
    support_path = Path(__file__).with_name("qwen35-agent-style-eval-support.py")
    spec = importlib.util.spec_from_file_location("qwen35_agent_style_eval_support", support_path)
    if spec is None or spec.loader is None:
        raise SystemExit(f"Could not load helper module: {support_path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--base-url", default="http://127.0.0.1:8004")
    parser.add_argument("--model", default="qwen3.5-4b")
    parser.add_argument("--cases", default="evals/qwen35-agent-style-eval-cases.json")
    parser.add_argument("--output", default="")
    parser.add_argument("--max-tokens", type=int, default=512)
    parser.add_argument("--temperature", type=float, default=0.0)
    parser.add_argument("--validate-only", action="store_true")
    args = parser.parse_args()
    support = load_support_module()

    cases_path = Path(args.cases)
    cases, manifest, scorecard = support.load_cases(cases_path)
    validation = support.validate_cases(cases, scorecard)
    if args.validate_only:
        print(
            json.dumps(
                {"ok": True, "validation": validation, "manifest": manifest, "scorecard": scorecard},
                ensure_ascii=False,
                indent=2,
            )
        )
        return

    results = []
    for case in cases:
        try:
            response = support.post_chat(args.base_url, args.model, case, args.max_tokens, args.temperature)
            failed_checks = support.run_checks(response["content"], case["checks"])
            dimension_results = support.evaluate_dimension_checks(response["content"], case)
            failed_dimensions = [dimension for dimension, result in dimension_results.items() if not result["ok"]]
            results.append(
                {
                    "id": case["id"],
                    "bucket": case["bucket"],
                    "ok": not failed_checks and not failed_dimensions,
                    "failed_checks": failed_checks,
                    "failed_dimensions": failed_dimensions,
                    "dimension_results": dimension_results,
                    "latency_s": round(response["latency_s"], 4),
                    "usage": response["usage"],
                    "content": response["content"],
                }
            )
        except (HTTPError, URLError, TimeoutError, json.JSONDecodeError, OSError) as exc:
            results.append(
                {
                    "id": case["id"],
                    "bucket": case["bucket"],
                    "ok": False,
                    "failed_checks": [f"request_error:{type(exc).__name__}"],
                    "failed_dimensions": [],
                    "dimension_results": {},
                    "latency_s": None,
                    "usage": {},
                    "content": "",
                    "error": str(exc),
                }
            )

    report = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "base_url": args.base_url,
        "model": args.model,
        "cases_file": str(cases_path),
        "manifest": manifest,
        "scorecard": scorecard,
        "validation": validation,
        "summary": support.summarize_with_scorecard(results, scorecard),
        "results": results,
    }
    output_path = Path(args.output) if args.output else Path("output/qwen35-agent-style-eval-report.json")
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(report["summary"], ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
