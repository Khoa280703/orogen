#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import time
from pathlib import Path
from typing import Any
from urllib.request import Request, urlopen


ALLOWED_CHECK_KEYS = {
    "exact",
    "contains_all",
    "contains_any",
    "not_contains",
    "regex",
    "max_chars",
    "max_words",
    "line_count",
    "unordered_line_regexes",
}


def load_jsonl(path: Path) -> list[dict[str, Any]]:
    cases: list[dict[str, Any]] = []
    try:
        lines = path.read_text().splitlines()
    except FileNotFoundError as exc:
        raise SystemExit(f"Bucket file not found: {path}") from exc
    for line_number, line in enumerate(lines, start=1):
        stripped = line.strip()
        if not stripped:
            continue
        try:
            payload = json.loads(stripped)
        except json.JSONDecodeError as exc:
            raise SystemExit(f"{path}:{line_number} contains invalid JSON: {exc}") from exc
        if not isinstance(payload, dict):
            raise SystemExit(f"{path}:{line_number} must be a JSON object.")
        cases.append(payload)
    return cases


def load_json_file(path: Path, label: str) -> dict[str, Any]:
    try:
        raw_text = path.read_text()
    except FileNotFoundError as exc:
        raise SystemExit(f"{label} not found: {path}") from exc
    try:
        payload = json.loads(raw_text)
    except json.JSONDecodeError as exc:
        raise SystemExit(f"{label} contains invalid JSON: {path}: {exc}") from exc
    if not isinstance(payload, dict):
        raise SystemExit(f"{label} must be a JSON object: {path}")
    return payload


def load_scorecard(path: Path) -> dict[str, Any]:
    payload = load_json_file(path, "Scorecard schema")
    dimensions = payload.get("dimensions")
    bucket_dimensions = payload.get("bucket_dimensions")
    if payload.get("format") != "qwen35-agent-style-scorecard-v1":
        raise SystemExit("Invalid scorecard schema format.")
    if not isinstance(dimensions, list) or not dimensions or any(not isinstance(item, str) or not item for item in dimensions):
        raise SystemExit("Scorecard schema is missing valid dimensions.")
    if not isinstance(bucket_dimensions, dict) or not bucket_dimensions:
        raise SystemExit("Scorecard schema is missing valid bucket_dimensions.")
    known_dimensions = set(dimensions)
    for bucket, mapped_dimensions in bucket_dimensions.items():
        if not isinstance(bucket, str) or not bucket.strip():
            raise SystemExit("bucket_dimensions contains an invalid bucket name.")
        if (
            not isinstance(mapped_dimensions, list)
            or not mapped_dimensions
            or any(not isinstance(item, str) or item not in known_dimensions for item in mapped_dimensions)
        ):
            raise SystemExit(f"bucket_dimensions for {bucket} is invalid.")
    return payload


def load_cases(path: Path) -> tuple[list[dict[str, Any]], dict[str, Any], dict[str, Any] | None]:
    if path.suffix != ".json":
        raise SystemExit("Manifest/case file must be a .json file.")
    try:
        payload = json.loads(path.read_text())
    except FileNotFoundError as exc:
        raise SystemExit(f"Case file not found: {path}") from exc
    except json.JSONDecodeError as exc:
        raise SystemExit(f"Case file contains invalid JSON: {path}: {exc}") from exc

    if isinstance(payload, dict) and payload.get("format") == "qwen35-agent-style-eval-manifest-v1":
        buckets = payload.get("buckets")
        if not isinstance(buckets, list) or not buckets:
            raise SystemExit("Manifest is missing valid buckets.")
        scorecard_path = payload.get("scorecard_path")
        scorecard = None
        if scorecard_path is not None:
            if not isinstance(scorecard_path, str) or not scorecard_path.strip():
                raise SystemExit("scorecard_path in manifest is invalid.")
            scorecard = load_scorecard((path.parent / scorecard_path).resolve())
        cases: list[dict[str, Any]] = []
        manifest_buckets: list[str] = []
        for bucket_entry in buckets:
            if not isinstance(bucket_entry, dict):
                raise SystemExit("Each manifest bucket entry must be an object.")
            bucket_name = bucket_entry.get("bucket")
            bucket_path = bucket_entry.get("path")
            if not isinstance(bucket_name, str) or not bucket_name.strip():
                raise SystemExit("Manifest bucket entry is missing a bucket name.")
            if not isinstance(bucket_path, str) or not bucket_path.strip():
                raise SystemExit("Manifest bucket entry is missing a path.")
            manifest_buckets.append(bucket_name)
            if scorecard and bucket_name not in scorecard.get("bucket_dimensions", {}):
                raise SystemExit(f"Bucket {bucket_name} exists in the manifest but is missing from the scorecard mapping.")
            resolved_path = (path.parent / bucket_path).resolve()
            if resolved_path.suffix != ".jsonl":
                raise SystemExit(f"Bucket file must be a .jsonl file: {resolved_path}")
            bucket_cases = load_jsonl(resolved_path)
            for case in bucket_cases:
                if case.get("bucket") != bucket_name:
                    raise SystemExit(f"Bucket mismatch in {resolved_path}: expected {bucket_name}, got {case.get('bucket')}")
            cases.extend(bucket_cases)
        if scorecard:
            scorecard_buckets = set(scorecard.get("bucket_dimensions", {}))
            extra_buckets = sorted(scorecard_buckets - set(manifest_buckets))
            if extra_buckets:
                raise SystemExit(f"Scorecard contains buckets not present in the manifest: {extra_buckets}")
        return cases, payload, scorecard

    if isinstance(payload, dict):
        if "cases" not in payload:
            raise SystemExit("Case file is missing top-level key `cases`.")
        cases = payload["cases"]
    else:
        cases = payload
    if not isinstance(cases, list) or not cases:
        raise SystemExit("Case file is invalid or empty.")
    return cases, {"format": "legacy-inline-cases"}, None


def validate_string_list(case_id: str, checks: dict[str, Any], key: str) -> None:
    value = checks.get(key)
    if value is None:
        return
    if not isinstance(value, list) or not value or any(not isinstance(item, str) or not item for item in value):
        raise SystemExit(f"Case {case_id} has invalid {key}; expected a non-empty list[str].")


def validate_checks(case_id: str, checks: dict[str, Any]) -> None:
    unknown_keys = set(checks) - ALLOWED_CHECK_KEYS
    if unknown_keys:
        raise SystemExit(f"Case {case_id} has unknown check keys: {sorted(unknown_keys)}")
    regex = checks.get("regex")
    exact = checks.get("exact")
    max_chars = checks.get("max_chars")
    max_words = checks.get("max_words")
    line_count = checks.get("line_count")
    if exact is not None and not isinstance(exact, str):
        raise SystemExit(f"Case {case_id} has invalid exact.")
    if regex is not None and not isinstance(regex, str):
        raise SystemExit(f"Case {case_id} has invalid regex.")
    if isinstance(regex, str):
        try:
            re.compile(regex)
        except re.error as exc:
            raise SystemExit(f"Case {case_id} has invalid regex pattern: {exc}") from exc
    if max_chars is not None and (not isinstance(max_chars, int) or max_chars <= 0):
        raise SystemExit(f"Case {case_id} has invalid max_chars.")
    if max_words is not None and (not isinstance(max_words, int) or max_words <= 0):
        raise SystemExit(f"Case {case_id} has invalid max_words.")
    if line_count is not None and (not isinstance(line_count, int) or line_count <= 0):
        raise SystemExit(f"Case {case_id} has invalid line_count.")
    validate_string_list(case_id, checks, "contains_all")
    validate_string_list(case_id, checks, "contains_any")
    validate_string_list(case_id, checks, "not_contains")
    unordered_line_regexes = checks.get("unordered_line_regexes")
    validate_string_list(case_id, checks, "unordered_line_regexes")
    for pattern in unordered_line_regexes or []:
        try:
            re.compile(pattern)
        except re.error as exc:
            raise SystemExit(f"Case {case_id} has invalid unordered_line_regexes pattern: {exc}") from exc


def validate_dimension_checks(
    case_id: str,
    bucket: str,
    dimension_checks: Any,
    scorecard: dict[str, Any] | None,
) -> list[str]:
    if scorecard:
        mapped_dimensions = scorecard.get("bucket_dimensions", {}).get(bucket, [])
        if dimension_checks is None:
            raise SystemExit(f"Case {case_id} is missing required dimension_checks for bucket {bucket}.")
    else:
        mapped_dimensions = []
        if dimension_checks is None:
            return []
    if not isinstance(dimension_checks, dict) or not dimension_checks:
        raise SystemExit(f"Case {case_id} has invalid dimension_checks.")

    allowed_dimensions: set[str] | None = None
    if scorecard:
        allowed_dimensions = set(mapped_dimensions)

    annotated_dimensions: list[str] = []
    for dimension, checks in dimension_checks.items():
        if not isinstance(dimension, str) or not dimension.strip():
            raise SystemExit(f"Case {case_id} has an invalid dimension name.")
        if allowed_dimensions is not None and dimension not in allowed_dimensions:
            raise SystemExit(f"Case {case_id} has dimension {dimension} that does not match the scorecard for bucket {bucket}.")
        if not isinstance(checks, dict) or not checks:
            raise SystemExit(f"Case {case_id} is missing checks for dimension {dimension}.")
        validate_checks(f"{case_id}:{dimension}", checks)
        annotated_dimensions.append(dimension)
    if allowed_dimensions is not None and set(annotated_dimensions) != allowed_dimensions:
        missing_dimensions = sorted(allowed_dimensions - set(annotated_dimensions))
        extra_dimensions = sorted(set(annotated_dimensions) - allowed_dimensions)
        details: list[str] = []
        if missing_dimensions:
            details.append(f"missing {missing_dimensions}")
        if extra_dimensions:
            details.append(f"extra {extra_dimensions}")
        raise SystemExit(f"Case {case_id} has incomplete dimension coverage for bucket {bucket}: {', '.join(details)}.")
    return annotated_dimensions


def validate_cases(cases: list[dict[str, Any]], scorecard: dict[str, Any] | None = None) -> dict[str, Any]:
    seen_ids: set[str] = set()
    buckets: dict[str, int] = {}
    dimension_coverage: dict[str, int] = {dimension: 0 for dimension in scorecard.get("dimensions", [])} if scorecard else {}
    for index, case in enumerate(cases, start=1):
        if not isinstance(case, dict):
            raise SystemExit(f"Case #{index} must be a JSON object.")
        case_id = case.get("id")
        bucket = case.get("bucket")
        prompt = case.get("prompt")
        checks = case.get("checks")
        dimension_checks = case.get("dimension_checks")
        if not isinstance(case_id, str) or not case_id.strip():
            raise SystemExit(f"Case #{index} is missing a valid id.")
        if case_id in seen_ids:
            raise SystemExit(f"Duplicate id: {case_id}")
        if not isinstance(bucket, str) or not bucket.strip():
            raise SystemExit(f"Case {case_id} is missing bucket.")
        if not isinstance(prompt, str) or not prompt.strip():
            raise SystemExit(f"Case {case_id} is missing prompt.")
        if not isinstance(checks, dict) or not checks:
            raise SystemExit(f"Case {case_id} is missing checks.")
        validate_checks(case_id, checks)
        for dimension in validate_dimension_checks(case_id, bucket, dimension_checks, scorecard):
            if dimension in dimension_coverage:
                dimension_coverage[dimension] += 1
        seen_ids.add(case_id)
        buckets[bucket] = buckets.get(bucket, 0) + 1
    return {"cases": len(cases), "buckets": buckets, "dimension_coverage": dimension_coverage}


def post_chat(base_url: str, model: str, case: dict[str, Any], max_tokens: int, temperature: float) -> dict[str, Any]:
    messages = []
    system_prompt = case.get("system_prompt")
    if isinstance(system_prompt, str) and system_prompt.strip():
        messages.append({"role": "system", "content": system_prompt})
    messages.append({"role": "user", "content": case["prompt"]})
    payload = {
        "model": model,
        "messages": messages,
        "temperature": temperature,
        "max_tokens": max_tokens,
        "stream": False,
        "chat_template_kwargs": {"enable_thinking": bool(case.get("enable_thinking", False))},
    }
    request = Request(
        f"{base_url.rstrip('/')}/v1/chat/completions",
        data=json.dumps(payload).encode("utf-8"),
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    started = time.perf_counter()
    with urlopen(request, timeout=1800) as response:
        body = json.loads(response.read().decode("utf-8"))
    elapsed = time.perf_counter() - started
    message = ((body.get("choices") or [{}])[0].get("message") or {})
    return {"content": (message.get("content") or "").strip(), "usage": body.get("usage") or {}, "latency_s": elapsed}


def run_checks(content: str, checks: dict[str, Any]) -> list[str]:
    failed: list[str] = []
    exact = checks.get("exact")
    if isinstance(exact, str) and content != exact:
        failed.append(f"exact != {exact}")
    contains_all = checks.get("contains_all") or []
    for item in contains_all:
        if item not in content:
            failed.append(f"missing:{item}")
    contains_any = checks.get("contains_any") or []
    if contains_any and not any(item in content for item in contains_any):
        failed.append("missing_any")
    not_contains = checks.get("not_contains") or []
    for item in not_contains:
        if item in content:
            failed.append(f"forbidden:{item}")
    regex = checks.get("regex")
    if isinstance(regex, str) and re.fullmatch(regex, content, flags=re.MULTILINE) is None:
        failed.append(f"regex:{regex}")
    max_chars = checks.get("max_chars")
    if isinstance(max_chars, int) and len(content) > max_chars:
        failed.append(f"too_long:{len(content)}>{max_chars}")
    max_words = checks.get("max_words")
    if isinstance(max_words, int):
        word_count = len(re.findall(r"\S+", content))
        if word_count > max_words:
            failed.append(f"too_many_words:{word_count}>{max_words}")
    line_count = checks.get("line_count")
    if isinstance(line_count, int) and len(content.splitlines()) != line_count:
        failed.append(f"line_count:{len(content.splitlines())}!={line_count}")
    unordered_line_regexes = checks.get("unordered_line_regexes") or []
    if unordered_line_regexes:
        lines = content.splitlines()
        used_indexes: set[int] = set()
        for pattern in unordered_line_regexes:
            matched_index = None
            for index, line in enumerate(lines):
                if index in used_indexes:
                    continue
                if re.search(pattern, line):
                    matched_index = index
                    break
            if matched_index is None:
                failed.append(f"missing_unordered_line_match:{pattern}")
            else:
                used_indexes.add(matched_index)
    return failed


def evaluate_dimension_checks(content: str, case: dict[str, Any]) -> dict[str, dict[str, Any]]:
    dimension_results: dict[str, dict[str, Any]] = {}
    for dimension, checks in (case.get("dimension_checks") or {}).items():
        failed_checks = run_checks(content, checks)
        dimension_results[dimension] = {"ok": not failed_checks, "failed_checks": failed_checks}
    return dimension_results


def summarize(results: list[dict[str, Any]]) -> dict[str, Any]:
    summary: dict[str, Any] = {"overall": {"cases": len(results), "pass": 0, "fail": 0}, "buckets": {}}
    for result in results:
        bucket = result["bucket"]
        bucket_summary = summary["buckets"].setdefault(bucket, {"cases": 0, "pass": 0, "fail": 0})
        bucket_summary["cases"] += 1
        if result["ok"]:
            summary["overall"]["pass"] += 1
            bucket_summary["pass"] += 1
        else:
            summary["overall"]["fail"] += 1
            bucket_summary["fail"] += 1
    summary["overall"]["pass_rate"] = round(summary["overall"]["pass"] / len(results), 4) if results else 0.0
    for bucket_summary in summary["buckets"].values():
        bucket_summary["pass_rate"] = round(bucket_summary["pass"] / bucket_summary["cases"], 4)
    return summary


def summarize_with_scorecard(results: list[dict[str, Any]], scorecard: dict[str, Any] | None) -> dict[str, Any]:
    summary = summarize(results)
    if not scorecard:
        return summary
    dimension_scores = {
        dimension: {"cases": 0, "pass": 0, "fail": 0, "scored_cases": 0, "proxy_cases": 0}
        for dimension in scorecard.get("dimensions", [])
    }
    dimension_proxies = {
        dimension: {"cases": 0, "pass": 0, "fail": 0} for dimension in scorecard.get("dimensions", [])
    }
    bucket_dimensions = scorecard.get("bucket_dimensions", {})
    for result in results:
        case_dimension_results = result.get("dimension_results") or {}
        for dimension in bucket_dimensions.get(result["bucket"], []):
            scored_dimension = dimension_scores[dimension]
            scored_dimension["cases"] += 1
            current_dimension_result = case_dimension_results.get(dimension)
            if isinstance(current_dimension_result, dict) and "ok" in current_dimension_result:
                passed = bool(current_dimension_result["ok"])
                scored_dimension["scored_cases"] += 1
            else:
                passed = bool(result["ok"])
                scored_dimension["proxy_cases"] += 1
            if passed:
                scored_dimension["pass"] += 1
            else:
                scored_dimension["fail"] += 1

            current = dimension_proxies[dimension]
            current["cases"] += 1
            if result["ok"]:
                current["pass"] += 1
            else:
                current["fail"] += 1
    for current in dimension_scores.values():
        current["pass_rate"] = round(current["pass"] / current["cases"], 4) if current["cases"] else 0.0
        current["coverage"] = round(current["scored_cases"] / current["cases"], 4) if current["cases"] else 0.0
    for current in dimension_proxies.values():
        current["pass_rate"] = round(current["pass"] / current["cases"], 4) if current["cases"] else 0.0
    summary["dimension_scores"] = {
        "source": "case-level dimension_checks; proxy fallback remains for other suites that do not enforce full mapped-dimension coverage",
        "values": dimension_scores,
    }
    summary["dimension_proxies"] = {
        "source": "bucket-to-dimension proxy from case-level pass/fail",
        "values": dimension_proxies,
    }
    return summary
