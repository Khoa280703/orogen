#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import threading
import time
from collections import deque
from dataclasses import asdict, dataclass
from http import HTTPStatus
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from itertools import count
from typing import Any
from urllib.error import HTTPError, URLError
from urllib.parse import urlparse
from urllib.request import Request, urlopen


HOST = os.environ.get("QWEN35_GATEWAY_HOST", "0.0.0.0")
PORT = int(os.environ.get("QWEN35_GATEWAY_PORT", "8004"))
REQUEST_TIMEOUT = int(os.environ.get("QWEN35_GATEWAY_REQUEST_TIMEOUT", "1800"))
QUEUE_WAIT_TIMEOUT = int(os.environ.get("QWEN35_GATEWAY_QUEUE_WAIT_TIMEOUT", "1800"))
HEALTH_CACHE_SECONDS = float(os.environ.get("QWEN35_GATEWAY_HEALTH_CACHE_SECONDS", "2"))
MODEL_NAME = os.environ.get("QWEN35_GATEWAY_MODEL", "qwen3.5-4b")
THINKING_MODEL_NAME = os.environ.get("QWEN35_GATEWAY_THINKING_MODEL", f"{MODEL_NAME}-thinking")
NON_THINKING_MODEL_NAME = os.environ.get("QWEN35_GATEWAY_NON_THINKING_MODEL", f"{MODEL_NAME}-no-thinking")
MAX_INPUT_TOKENS = int(os.environ.get("QWEN35_GATEWAY_MAX_INPUT_TOKENS", "128000"))
MAX_TOTAL_TOKENS = int(os.environ.get("QWEN35_GATEWAY_MAX_TOTAL_TOKENS", "131072"))
DEFAULT_MAX_OUTPUT_TOKENS = int(os.environ.get("QWEN35_GATEWAY_DEFAULT_MAX_OUTPUT_TOKENS", "2048"))
TOKEN_BUDGET_RESERVE = int(os.environ.get("QWEN35_GATEWAY_TOKEN_BUDGET_RESERVE", "0"))
MAX_LIVE_REQUESTS_PER_UPSTREAM = int(os.environ.get("QWEN35_GATEWAY_MAX_LIVE_REQUESTS_PER_UPSTREAM", "24"))
RAW_UPSTREAMS = os.environ.get(
    "QWEN35_GATEWAY_UPSTREAMS",
    "http://127.0.0.1:8100,http://127.0.0.1:8101,http://127.0.0.1:8102",
)
RAW_UPSTREAM_BUDGETS = os.environ.get("QWEN35_GATEWAY_UPSTREAM_BUDGETS", "")
RAW_UPSTREAM_MAX_LIVE_REQUESTS = os.environ.get("QWEN35_GATEWAY_UPSTREAM_MAX_LIVE_REQUESTS", "")
UPSTREAMS = [item.strip().rstrip("/") for item in RAW_UPSTREAMS.split(",") if item.strip()]
RR_COUNTER = count()


@dataclass
class UpstreamState:
    inflight: int = 0
    reserved_tokens: int = 0
    token_budget: int = 0
    max_live_requests: int = MAX_LIVE_REQUESTS_PER_UPSTREAM

    @property
    def remaining_tokens(self) -> int:
        return self.token_budget - self.reserved_tokens


STATE_LOCK = threading.Lock()
STATE_COND = threading.Condition(STATE_LOCK)
UPSTREAM_STATE: dict[str, UpstreamState] = {}
WAIT_QUEUE: deque[int] = deque()
WAITERS: dict[int, dict[str, Any]] = {}
HEALTH_CACHE: dict[str, tuple[bool, float]] = {}
REQUEST_IDS = count(1)
MODEL_ALIAS_BEHAVIORS = {
    MODEL_NAME: None,
    THINKING_MODEL_NAME: True,
    NON_THINKING_MODEL_NAME: False,
}


def _default_budget_for_upstream(upstream: str) -> int:
    parsed = urlparse(upstream)
    if parsed.port == 8100:
        return 300000
    return 400000


def _default_max_live_requests_for_upstream(upstream: str) -> int:
    parsed = urlparse(upstream)
    if parsed.port == 8100:
        return 18
    return MAX_LIVE_REQUESTS_PER_UPSTREAM


def _parse_upstream_int_overrides(raw_values: str) -> dict[str, int]:
    overrides: dict[str, int] = {}
    for raw_item in raw_values.split(","):
        item = raw_item.strip()
        if not item or "=" not in item:
            continue
        upstream, raw_value = item.split("=", 1)
        upstream = upstream.strip().rstrip("/")
        try:
            overrides[upstream] = int(raw_value.strip())
        except ValueError:
            continue
    return overrides


for upstream in UPSTREAMS:
    UPSTREAM_STATE[upstream] = UpstreamState(
        token_budget=_default_budget_for_upstream(upstream),
        max_live_requests=_default_max_live_requests_for_upstream(upstream),
    )

for upstream, budget in _parse_upstream_int_overrides(RAW_UPSTREAM_BUDGETS).items():
    if upstream in UPSTREAM_STATE:
        UPSTREAM_STATE[upstream].token_budget = budget

for upstream, max_live_requests in _parse_upstream_int_overrides(RAW_UPSTREAM_MAX_LIVE_REQUESTS).items():
    if upstream in UPSTREAM_STATE:
        UPSTREAM_STATE[upstream].max_live_requests = max(1, max_live_requests)


def _is_streaming(payload: dict[str, Any], accept_header: str) -> bool:
    return bool(payload.get("stream")) or "text/event-stream" in accept_header.lower()


def _request(url: str, method: str, payload: dict[str, Any] | None, timeout: int) -> tuple[int, bytes, str]:
    headers = {"Content-Type": "application/json"}
    body = None if payload is None else json.dumps(payload).encode("utf-8")
    request = Request(url, headers=headers, data=body, method=method)
    with urlopen(request, timeout=timeout) as response:
        return response.status, response.read(), response.headers.get_content_type()


def _request_json(url: str, method: str, payload: dict[str, Any] | None, timeout: int) -> tuple[int, dict[str, Any]]:
    status, data, _ = _request(url, method, payload, timeout)
    return status, json.loads(data.decode("utf-8"))


def _is_upstream_healthy(upstream: str) -> bool:
    now = time.monotonic()
    cached = HEALTH_CACHE.get(upstream)
    if cached and (now - cached[1]) < HEALTH_CACHE_SECONDS:
        return cached[0]

    healthy = False
    try:
        status, _, _ = _request(f"{upstream}/health", "GET", None, timeout=5)
        healthy = status == HTTPStatus.OK
    except Exception:
        healthy = False

    HEALTH_CACHE[upstream] = (healthy, now)
    return healthy


def _healthy_upstreams() -> list[str]:
    return [upstream for upstream in UPSTREAMS if _is_upstream_healthy(upstream)]


def _choose_proxy_upstream() -> str:
    healthy = _healthy_upstreams()
    if not healthy:
        raise RuntimeError("Khong con replica 4B nao healthy.")

    rr_seed = next(RR_COUNTER)
    ordered = sorted(
        healthy,
        key=lambda upstream: (
            UPSTREAM_STATE[upstream].inflight,
            -UPSTREAM_STATE[upstream].remaining_tokens,
            (healthy.index(upstream) - rr_seed) % len(healthy),
        ),
    )
    return ordered[0]


def _extract_max_output_tokens(payload: dict[str, Any] | None) -> int:
    if not isinstance(payload, dict):
        return DEFAULT_MAX_OUTPUT_TOKENS

    for key in ("max_tokens", "max_output_tokens"):
        value = payload.get(key)
        if isinstance(value, int) and value >= 0:
            return value

    if isinstance(payload.get("max_tokens"), str):
        try:
            return max(0, int(payload["max_tokens"]))
        except ValueError:
            return DEFAULT_MAX_OUTPUT_TOKENS

    return DEFAULT_MAX_OUTPUT_TOKENS


def _normalize_model_alias(payload: dict[str, Any] | None) -> str | None:
    if not isinstance(payload, dict):
        return None

    requested_model = payload.get("model", MODEL_NAME)
    if not isinstance(requested_model, str):
        requested_model = MODEL_NAME

    force_thinking = MODEL_ALIAS_BEHAVIORS.get(requested_model)
    payload["model"] = MODEL_NAME

    if force_thinking is None:
        return requested_model

    chat_template_kwargs = payload.get("chat_template_kwargs")
    if not isinstance(chat_template_kwargs, dict):
        chat_template_kwargs = {}
        payload["chat_template_kwargs"] = chat_template_kwargs

    chat_template_kwargs["enable_thinking"] = force_thinking
    return requested_model


def _build_models_response(upstream_payload: dict[str, Any]) -> dict[str, Any]:
    models = upstream_payload.get("data")
    if not isinstance(models, list) or not models:
        return {
            "object": "list",
            "data": [],
        }

    base_model = models[0]
    response_models = []
    for alias in (THINKING_MODEL_NAME, NON_THINKING_MODEL_NAME, MODEL_NAME):
        alias_model = dict(base_model)
        alias_model["id"] = alias
        response_models.append(alias_model)

    return {
        "object": upstream_payload.get("object", "list"),
        "data": response_models,
    }


def _build_count_tokens_payload(payload: dict[str, Any] | None) -> dict[str, Any] | None:
    if not isinstance(payload, dict):
        return None

    if isinstance(payload.get("messages"), list):
        count_payload = {
            "model": payload.get("model", MODEL_NAME),
            "messages": payload["messages"],
        }
        system = payload.get("system")
        if system is not None:
            count_payload["system"] = system
        return count_payload

    prompt = payload.get("prompt")
    if isinstance(prompt, str):
        return {
            "model": payload.get("model", MODEL_NAME),
            "messages": [{"role": "user", "content": prompt}],
        }

    user_input = payload.get("input")
    if isinstance(user_input, str):
        return {
            "model": payload.get("model", MODEL_NAME),
            "messages": [{"role": "user", "content": user_input}],
        }

    return None


def _count_input_tokens(payload: dict[str, Any] | None) -> int:
    count_payload = _build_count_tokens_payload(payload)
    if count_payload is None:
        return 0

    last_error: Exception | None = None
    for upstream in _healthy_upstreams():
        try:
            _, data = _request_json(
                f"{upstream}/v1/messages/count_tokens",
                "POST",
                count_payload,
                timeout=REQUEST_TIMEOUT,
            )
            input_tokens = data.get("input_tokens")
            if isinstance(input_tokens, int):
                return input_tokens
        except Exception as exc:  # noqa: BLE001
            last_error = exc
            continue

    if last_error is not None:
        raise last_error
    raise RuntimeError("Khong dem duoc token cho request.")


def _validate_request_size(input_tokens: int, max_output_tokens: int) -> tuple[bool, dict[str, Any] | None]:
    if input_tokens > MAX_INPUT_TOKENS:
        return False, {
            "error": "Input vuot gioi han context cong khai.",
            "input_tokens": input_tokens,
            "max_input_tokens": MAX_INPUT_TOKENS,
        }

    total_tokens = input_tokens + max_output_tokens
    if total_tokens > MAX_TOTAL_TOKENS:
        return False, {
            "error": "Tong input va output du kien vuot gioi han model.",
            "input_tokens": input_tokens,
            "max_output_tokens": max_output_tokens,
            "max_total_tokens": MAX_TOTAL_TOKENS,
        }

    return True, None


def _reserve_tokens_for_request(path: str, payload: dict[str, Any] | None) -> tuple[str, int, int]:
    if path not in {"/v1/chat/completions", "/v1/completions", "/v1/messages", "/v1/responses"}:
        return _choose_proxy_upstream(), 0, 0

    input_tokens = _count_input_tokens(payload)
    max_output_tokens = _extract_max_output_tokens(payload)
    is_valid, error_payload = _validate_request_size(input_tokens, max_output_tokens)
    if not is_valid:
        raise ValueError(json.dumps(error_payload, ensure_ascii=False))

    required_tokens = input_tokens + TOKEN_BUDGET_RESERVE
    request_id = next(REQUEST_IDS)
    deadline = time.monotonic() + QUEUE_WAIT_TIMEOUT

    with STATE_COND:
        WAIT_QUEUE.append(request_id)
        WAITERS[request_id] = {
            "input_tokens": input_tokens,
            "required_tokens": required_tokens,
            "max_output_tokens": max_output_tokens,
        }
        while True:
            if request_id not in WAITERS:
                raise RuntimeError("Yeu cau da roi khoi hang doi.")

            if WAIT_QUEUE and WAIT_QUEUE[0] == request_id:
                healthy = _healthy_upstreams()
                fitting = [
                    upstream
                    for upstream in healthy
                    if UPSTREAM_STATE[upstream].remaining_tokens >= required_tokens
                    and UPSTREAM_STATE[upstream].inflight < UPSTREAM_STATE[upstream].max_live_requests
                ]
                if fitting:
                    upstream = max(
                        fitting,
                        key=lambda item: (
                            UPSTREAM_STATE[item].remaining_tokens,
                            -UPSTREAM_STATE[item].inflight,
                        ),
                    )
                    WAIT_QUEUE.popleft()
                    WAITERS.pop(request_id, None)
                    UPSTREAM_STATE[upstream].inflight += 1
                    UPSTREAM_STATE[upstream].reserved_tokens += required_tokens
                    return upstream, required_tokens, input_tokens

            remaining = deadline - time.monotonic()
            if remaining <= 0:
                try:
                    WAIT_QUEUE.remove(request_id)
                except ValueError:
                    pass
                WAITERS.pop(request_id, None)
                raise TimeoutError("Hang doi token-budget qua lau, khong co GPU nao du room.")

            STATE_COND.wait(timeout=min(1.0, remaining))


def _release_upstream(upstream: str, reserved_tokens: int) -> None:
    if upstream not in UPSTREAM_STATE:
        return
    with STATE_COND:
        state = UPSTREAM_STATE[upstream]
        state.inflight = max(0, state.inflight - 1)
        state.reserved_tokens = max(0, state.reserved_tokens - reserved_tokens)
        STATE_COND.notify_all()


class GatewayHandler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def _route_path(self) -> str:
        return urlparse(self.path).path

    def do_GET(self) -> None:
        route_path = self._route_path()
        if route_path in {"/", "/health"}:
            self._send_health()
            return
        self._proxy("GET")

    def do_HEAD(self) -> None:
        route_path = self._route_path()
        if route_path in {"/", "/health"}:
            self.send_response(HTTPStatus.OK if _healthy_upstreams() else HTTPStatus.BAD_GATEWAY)
            self.end_headers()
            return
        self._proxy("HEAD")

    def do_POST(self) -> None:
        self._proxy("POST")

    def log_message(self, fmt: str, *args: Any) -> None:
        print(f"[qwen35-4b-gateway] {self.address_string()} - {fmt % args}")

    def end_headers(self) -> None:
        self.send_header("Cache-Control", "no-store")
        super().end_headers()

    def _send_health(self) -> None:
        healthy = _healthy_upstreams()
        queue_size = len(WAIT_QUEUE)
        upstream_state = {upstream: asdict(state) | {"remaining_tokens": state.remaining_tokens} for upstream, state in UPSTREAM_STATE.items()}
        status = HTTPStatus.OK if healthy else HTTPStatus.BAD_GATEWAY
        self._send_json(
            {
                "ok": bool(healthy),
                "model": MODEL_NAME,
                "healthy_upstreams": healthy,
                "queue_size": queue_size,
                "max_input_tokens": MAX_INPUT_TOKENS,
                "max_total_tokens": MAX_TOTAL_TOKENS,
                "upstreams": upstream_state,
            },
            status=status,
        )

    def _proxy(self, method: str) -> None:
        route_path = self._route_path()

        if route_path not in {
            "/v1/models",
            "/v1/chat/completions",
            "/v1/completions",
            "/v1/messages",
            "/v1/messages/count_tokens",
            "/v1/responses",
        }:
            self._send_json({"error": "Not found"}, status=HTTPStatus.NOT_FOUND)
            return

        payload = None
        if method == "POST":
            length = int(self.headers.get("Content-Length", "0"))
            raw_body = self.rfile.read(length)
            try:
                payload = json.loads(raw_body.decode("utf-8"))
            except json.JSONDecodeError:
                self._send_json({"error": "Body JSON khong hop le."}, status=HTTPStatus.BAD_REQUEST)
                return
            if isinstance(payload, dict):
                payload.setdefault("model", MODEL_NAME)
                _normalize_model_alias(payload)

        if route_path == "/v1/models":
            try:
                upstream = _choose_proxy_upstream()
                _, upstream_payload = _request_json(f"{upstream}/v1/models", "GET", None, timeout=REQUEST_TIMEOUT)
                self._send_json(_build_models_response(upstream_payload))
            except Exception as exc:  # noqa: BLE001
                self._send_json(
                    {"error": "Khong lay duoc danh sach model.", "detail": str(exc)},
                    status=HTTPStatus.BAD_GATEWAY,
                )
            return

        accept_header = self.headers.get("Accept", "")
        streaming = isinstance(payload, dict) and _is_streaming(payload, accept_header)
        scheduled = route_path not in {"/v1/messages/count_tokens"}
        upstream = ""
        reserved_tokens = 0
        input_tokens = 0

        try:
            if scheduled:
                upstream, reserved_tokens, input_tokens = _reserve_tokens_for_request(route_path, payload)
            else:
                upstream = _choose_proxy_upstream()
        except ValueError as exc:
            self._send_json(json.loads(str(exc)), status=HTTPStatus.BAD_REQUEST)
            return
        except TimeoutError as exc:
            self._send_json(
                {
                    "error": str(exc),
                    "queue_size": len(WAIT_QUEUE),
                },
                status=HTTPStatus.SERVICE_UNAVAILABLE,
            )
            return
        except RuntimeError as exc:
            self._send_json({"error": str(exc)}, status=HTTPStatus.BAD_GATEWAY)
            return
        except Exception as exc:  # noqa: BLE001
            self._send_json({"error": "Khong reserve duoc token budget.", "detail": str(exc)}, status=HTTPStatus.BAD_GATEWAY)
            return

        try:
            if streaming:
                self._proxy_stream(upstream, method, payload)
            else:
                self._proxy_json(upstream, method, payload, input_tokens=input_tokens, reserved_tokens=reserved_tokens)
        finally:
            if scheduled:
                _release_upstream(upstream, reserved_tokens)

    def _proxy_json(
        self,
        upstream: str,
        method: str,
        payload: dict[str, Any] | None,
        *,
        input_tokens: int,
        reserved_tokens: int,
    ) -> None:
        url = f"{upstream}{self.path}"
        try:
            status, data, content_type = _request(url, method, payload, timeout=REQUEST_TIMEOUT)
            self.send_response(status)
            self.send_header("Content-Type", content_type)
            self.send_header("Content-Length", str(len(data)))
            if reserved_tokens:
                self.send_header("X-Qwen-Gateway-Upstream", upstream)
                self.send_header("X-Qwen-Gateway-Reserved-Tokens", str(reserved_tokens))
                self.send_header("X-Qwen-Gateway-Input-Tokens", str(input_tokens))
            self.end_headers()
            self.wfile.write(data)
        except HTTPError as exc:
            detail = exc.read() or json.dumps({"error": str(exc)}).encode("utf-8")
            self.send_response(exc.code)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(detail)))
            self.end_headers()
            self.wfile.write(detail)
        except URLError as exc:
            self._send_json(
                {"error": "Khong ket noi duoc toi replica.", "detail": str(exc.reason), "upstream": upstream},
                status=HTTPStatus.BAD_GATEWAY,
            )

    def _proxy_stream(self, upstream: str, method: str, payload: dict[str, Any]) -> None:
        url = f"{upstream}{self.path}"
        headers = {"Content-Type": "application/json", "Accept": "text/event-stream"}
        request = Request(url, headers=headers, data=json.dumps(payload).encode("utf-8"), method=method)
        try:
            with urlopen(request, timeout=REQUEST_TIMEOUT) as response:
                self.send_response(response.status)
                self.send_header("Content-Type", "text/event-stream; charset=utf-8")
                self.send_header("Connection", "close")
                self.send_header("X-Accel-Buffering", "no")
                self.end_headers()
                while True:
                    line = response.readline()
                    if not line:
                        break
                    try:
                        self.wfile.write(line)
                        self.wfile.flush()
                    except (BrokenPipeError, ConnectionResetError):
                        break
        except HTTPError as exc:
            detail = exc.read() or json.dumps({"error": str(exc)}).encode("utf-8")
            self.send_response(exc.code)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(detail)))
            self.end_headers()
            self.wfile.write(detail)
        except URLError as exc:
            self._send_json(
                {"error": "Khong ket noi duoc toi replica.", "detail": str(exc.reason), "upstream": upstream},
                status=HTTPStatus.BAD_GATEWAY,
            )

    def _send_json(self, payload: dict[str, Any], status: int = HTTPStatus.OK) -> None:
        data = json.dumps(payload, ensure_ascii=False).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)


def main() -> None:
    if not UPSTREAMS:
        raise SystemExit("Thieu QWEN35_GATEWAY_UPSTREAMS.")
    server = ThreadingHTTPServer((HOST, PORT), GatewayHandler)
    budgets = ", ".join(f"{upstream}={UPSTREAM_STATE[upstream].token_budget}" for upstream in UPSTREAMS)
    print(f"==> Qwen 4B gateway: http://{HOST}:{PORT}")
    print(f"==> Upstreams: {', '.join(UPSTREAMS)}")
    print(f"==> Token budgets: {budgets}")
    server.serve_forever()


if __name__ == "__main__":
    main()
