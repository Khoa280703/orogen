#!/usr/bin/env python3
from __future__ import annotations

import json
import os
from http import HTTPStatus
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen


ROOT_DIR = Path(__file__).resolve().parent.parent
STATIC_DIR = ROOT_DIR / "ui" / "qwen35-chat-ui"
VLLM_BASE_URL = os.environ.get("QWEN35_UI_VLLM_BASE_URL", "http://127.0.0.1:8002").rstrip("/")
HOST = os.environ.get("QWEN35_UI_HOST", "0.0.0.0")
PORT = int(os.environ.get("QWEN35_UI_PORT", "8010"))
API_KEY = os.environ.get("QWEN35_UI_API_KEY", "")
REQUEST_TIMEOUT = int(os.environ.get("QWEN35_UI_REQUEST_TIMEOUT", "1800"))
DEFAULT_MODEL = os.environ.get("QWEN35_UI_DEFAULT_MODEL", "qwen3.5-27b")


class ChatUiHandler(SimpleHTTPRequestHandler):
    def __init__(self, *args: Any, **kwargs: Any) -> None:
        super().__init__(*args, directory=str(STATIC_DIR), **kwargs)

    def do_GET(self) -> None:
        if self.path == "/api/config":
            self._send_json(
                {
                    "upstreamBaseUrl": VLLM_BASE_URL,
                    "defaultModel": DEFAULT_MODEL,
                }
            )
            return
        if self.path == "/api/models":
            self._proxy_models()
            return
        if self.path in {"/", "/index.html"}:
            self.path = "/index.html"
        return super().do_GET()

    def do_POST(self) -> None:
        if self.path == "/api/chat":
            self._proxy_chat()
            return
        self._send_json({"error": "Not found"}, status=HTTPStatus.NOT_FOUND)

    def log_message(self, format: str, *args: Any) -> None:
        print(f"[qwen35-chat-ui] {self.address_string()} - {format % args}")

    def end_headers(self) -> None:
        self.send_header("Cache-Control", "no-store")
        super().end_headers()

    def _proxy_models(self) -> None:
        self._proxy_json("GET", "/v1/models", None)

    def _proxy_chat(self) -> None:
        length = int(self.headers.get("Content-Length", "0"))
        raw_body = self.rfile.read(length)
        try:
            payload = json.loads(raw_body.decode("utf-8"))
        except json.JSONDecodeError:
            self._send_json({"error": "Body JSON không hợp lệ."}, status=HTTPStatus.BAD_REQUEST)
            return

        if not payload.get("messages"):
            self._send_json({"error": "Thiếu messages."}, status=HTTPStatus.BAD_REQUEST)
            return

        if payload.get("stream"):
            self._proxy_stream("POST", "/v1/chat/completions", payload)
            return

        payload.setdefault("stream", False)
        self._proxy_json("POST", "/v1/chat/completions", payload)

    def _proxy_json(self, method: str, api_path: str, payload: dict[str, Any] | None) -> None:
        url = f"{VLLM_BASE_URL}{api_path}"
        headers = {"Content-Type": "application/json"}
        if API_KEY:
            headers["Authorization"] = f"Bearer {API_KEY}"

        body = None if payload is None else json.dumps(payload).encode("utf-8")
        request = Request(url, data=body, headers=headers, method=method)

        try:
            with urlopen(request, timeout=REQUEST_TIMEOUT) as response:
                data = response.read()
                self.send_response(response.status)
                self.send_header("Content-Type", response.headers.get_content_type())
                self.end_headers()
                self.wfile.write(data)
        except HTTPError as exc:
            detail = exc.read() or json.dumps({"error": str(exc)}).encode("utf-8")
            self.send_response(exc.code)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(detail)
        except URLError as exc:
            self._send_json(
                {
                    "error": "Không kết nối được tới vLLM.",
                    "detail": str(exc.reason),
                    "base_url": VLLM_BASE_URL,
                },
                status=HTTPStatus.BAD_GATEWAY,
            )

    def _proxy_stream(self, method: str, api_path: str, payload: dict[str, Any]) -> None:
        url = f"{VLLM_BASE_URL}{api_path}"
        headers = {
            "Content-Type": "application/json",
            "Accept": "text/event-stream",
        }
        if API_KEY:
            headers["Authorization"] = f"Bearer {API_KEY}"

        request = Request(
            url,
            data=json.dumps(payload).encode("utf-8"),
            headers=headers,
            method=method,
        )

        try:
            with urlopen(request, timeout=REQUEST_TIMEOUT) as response:
                self.send_response(response.status)
                self.send_header("Content-Type", "text/event-stream; charset=utf-8")
                self.send_header("Cache-Control", "no-store")
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
                {
                    "error": "Không kết nối được tới vLLM.",
                    "detail": str(exc.reason),
                    "base_url": VLLM_BASE_URL,
                },
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
    if not STATIC_DIR.joinpath("index.html").exists():
        raise SystemExit(f"Thiếu UI ở {STATIC_DIR}")

    server = ThreadingHTTPServer((HOST, PORT), ChatUiHandler)
    print(f"==> Qwen chat UI: http://{HOST}:{PORT}")
    print(f"==> Proxying to: {VLLM_BASE_URL}")
    server.serve_forever()


if __name__ == "__main__":
    main()
