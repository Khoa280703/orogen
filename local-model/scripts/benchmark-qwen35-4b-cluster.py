#!/usr/bin/env python3
from __future__ import annotations

import argparse
import concurrent.futures
import json
import statistics
import time
from pathlib import Path
from typing import Any
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen


def post_json(base_url: str, path: str, payload: dict[str, Any], timeout: int = 1800) -> dict[str, Any]:
    request = Request(
        f"{base_url.rstrip('/')}{path}",
        data=json.dumps(payload).encode("utf-8"),
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    with urlopen(request, timeout=timeout) as response:
        return json.loads(response.read().decode("utf-8"))


def get_json(base_url: str, path: str, timeout: int = 60) -> dict[str, Any]:
    with urlopen(f"{base_url.rstrip('/')}{path}", timeout=timeout) as response:
        return json.loads(response.read().decode("utf-8"))


def build_prompt(base_url: str, model: str, target_tokens: int) -> tuple[str, int]:
    chunk = "def helper(value): return value * 2 # benchmark context block\n"
    lo, hi = 1, max(4, target_tokens // 8)
    best_text, best_tokens = chunk, 0
    while lo <= hi:
        mid = (lo + hi) // 2
        content = chunk * mid
        payload = {"model": model, "messages": [{"role": "user", "content": content}]}
        count = post_json(base_url, "/v1/messages/count_tokens", payload)["input_tokens"]
        if count <= target_tokens:
            best_text, best_tokens = content, count
            lo = mid + 1
        else:
            hi = mid - 1
    return best_text, best_tokens


def run_chat(base_url: str, model: str, prompt: str, max_tokens: int, stream: bool) -> dict[str, Any]:
    payload = {
        "model": model,
        "messages": [{"role": "user", "content": prompt}],
        "max_tokens": max_tokens,
        "temperature": 0,
        "stream": stream,
        "chat_template_kwargs": {"enable_thinking": False},
    }
    started = time.perf_counter()
    if not stream:
        try:
            data = post_json(base_url, "/v1/chat/completions", payload)
            elapsed = time.perf_counter() - started
            usage = data.get("usage") or {}
            return {
                "ok": True,
                "latency_s": elapsed,
                "prompt_tokens": usage.get("prompt_tokens", 0),
                "completion_tokens": usage.get("completion_tokens", 0),
                "total_tokens": usage.get("total_tokens", 0),
            }
        except HTTPError as exc:
            return {
                "ok": False,
                "latency_s": time.perf_counter() - started,
                "error": f"HTTP {exc.code}",
                "detail": exc.read().decode("utf-8", errors="replace"),
            }
        except URLError as exc:
            return {
                "ok": False,
                "latency_s": time.perf_counter() - started,
                "error": "URLError",
                "detail": str(exc.reason),
            }
        except Exception as exc:  # noqa: BLE001
            return {
                "ok": False,
                "latency_s": time.perf_counter() - started,
                "error": type(exc).__name__,
                "detail": str(exc),
            }

    request = Request(
        f"{base_url.rstrip('/')}/v1/chat/completions",
        data=json.dumps(payload).encode("utf-8"),
        headers={"Content-Type": "application/json", "Accept": "text/event-stream"},
        method="POST",
    )
    first_token_s = None
    completion_tokens = 0
    prompt_tokens = 0
    try:
        with urlopen(request, timeout=1800) as response:
            while True:
                line = response.readline()
                if not line:
                    break
                if not line.startswith(b"data: "):
                    continue
                body = line[6:].strip()
                if body == b"[DONE]":
                    break
                event = json.loads(body.decode("utf-8"))
                if first_token_s is None:
                    first_token_s = time.perf_counter() - started
                choice = (event.get("choices") or [{}])[0]
                delta = choice.get("delta") or {}
                if delta.get("content"):
                    completion_tokens += 1
                usage = event.get("usage") or {}
                prompt_tokens = usage.get("prompt_tokens", prompt_tokens)
        return {
            "ok": True,
            "latency_s": time.perf_counter() - started,
            "first_token_s": first_token_s or 0.0,
            "prompt_tokens": prompt_tokens,
            "completion_tokens": completion_tokens,
            "total_tokens": prompt_tokens + completion_tokens,
        }
    except HTTPError as exc:
        return {
            "ok": False,
            "latency_s": time.perf_counter() - started,
            "error": f"HTTP {exc.code}",
            "detail": exc.read().decode("utf-8", errors="replace"),
        }
    except URLError as exc:
        return {
            "ok": False,
            "latency_s": time.perf_counter() - started,
            "error": "URLError",
            "detail": str(exc.reason),
        }
    except Exception as exc:  # noqa: BLE001
        return {
            "ok": False,
            "latency_s": time.perf_counter() - started,
            "error": type(exc).__name__,
            "detail": str(exc),
        }


def summarize(results: list[dict[str, Any]]) -> dict[str, Any]:
    latencies = [item["latency_s"] for item in results if item.get("ok")]
    total_tokens = sum(item.get("total_tokens", 0) for item in results if item.get("ok"))
    duration = sum(latencies) if latencies else 0.0
    errors = [item.get("error") for item in results if not item.get("ok")]
    return {
        "requests": len(results),
        "success": sum(1 for item in results if item.get("ok")),
        "fail": sum(1 for item in results if not item.get("ok")),
        "p50_latency_s": statistics.median(latencies) if latencies else None,
        "p95_latency_s": statistics.quantiles(latencies, n=20)[18] if len(latencies) >= 20 else max(latencies, default=None),
        "avg_latency_s": statistics.mean(latencies) if latencies else None,
        "aggregate_tokens": total_tokens,
        "avg_tokens_per_s": (total_tokens / duration) if duration else None,
        "errors": errors[:5],
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--base-url", default="http://127.0.0.1:8004")
    parser.add_argument("--model", default="qwen3.5-4b")
    parser.add_argument("--output", default="")
    args = parser.parse_args()

    model_info = get_json(args.base_url, "/v1/models")
    short_prompt, short_tokens = build_prompt(args.base_url, args.model, 128)
    medium_prompt, medium_tokens = build_prompt(args.base_url, args.model, 2048)
    long_prompt, long_tokens = build_prompt(args.base_url, args.model, 8192)

    sequential = {}
    for name, prompt, max_tokens in [
        ("short", short_prompt, 128),
        ("medium", medium_prompt, 256),
        ("long", long_prompt, 256),
    ]:
        runs = [run_chat(args.base_url, args.model, prompt, max_tokens, False) for _ in range(3)]
        sequential[name] = summarize(runs) | {"prompt_tokens_target": {"short": short_tokens, "medium": medium_tokens, "long": long_tokens}[name]}

    stream_runs = [run_chat(args.base_url, args.model, short_prompt, 128, True) for _ in range(3)]
    streaming = summarize(stream_runs) | {
        "avg_first_token_s": statistics.mean(item["first_token_s"] for item in stream_runs),
        "p50_first_token_s": statistics.median(item["first_token_s"] for item in stream_runs),
    }

    load = {}
    for concurrency in (3, 6, 9, 12, 18):
        started = time.perf_counter()
        with concurrent.futures.ThreadPoolExecutor(max_workers=concurrency) as pool:
            futures = [pool.submit(run_chat, args.base_url, args.model, medium_prompt, 128, False) for _ in range(concurrency * 2)]
            results = [future.result() for future in concurrent.futures.as_completed(futures)]
        elapsed = time.perf_counter() - started
        load[str(concurrency)] = summarize(results) | {"wall_time_s": elapsed, "req_per_s": len(results) / elapsed}

    report = {
        "base_url": args.base_url,
        "model": args.model,
        "models_endpoint": model_info,
        "prompt_tokens": {"short": short_tokens, "medium": medium_tokens, "long": long_tokens},
        "sequential": sequential,
        "streaming_short": streaming,
        "load_medium_prompt": load,
    }

    output_path = Path(args.output) if args.output else Path("output/qwen35-4b-benchmark-report.json")
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(report, ensure_ascii=False, indent=2))
    print(json.dumps(report, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    try:
        main()
    except (HTTPError, URLError) as exc:
        raise SystemExit(f"Benchmark fail: {exc}") from exc
