#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
UI_HOST="${QWEN35_UI_HOST:-0.0.0.0}"
UI_PORT="${QWEN35_UI_PORT:-8010}"
VLLM_BASE_URL="${QWEN35_UI_VLLM_BASE_URL:-http://127.0.0.1:8002}"
PYTHON_BIN="${QWEN35_UI_PYTHON_BIN:-python3}"

if ! command -v "$PYTHON_BIN" >/dev/null 2>&1; then
  echo "Không tìm thấy Python: $PYTHON_BIN" >&2
  exit 1
fi

if ! curl -fsS "${VLLM_BASE_URL}/v1/models" >/dev/null 2>&1; then
  echo "Không chạm được vLLM ở ${VLLM_BASE_URL}. Kiểm tra lại service model trước." >&2
  exit 1
fi

echo "==> Starting Qwen chat UI on http://${UI_HOST}:${UI_PORT}"
echo "==> Using upstream ${VLLM_BASE_URL}"

exec env \
  QWEN35_UI_HOST="$UI_HOST" \
  QWEN35_UI_PORT="$UI_PORT" \
  QWEN35_UI_VLLM_BASE_URL="$VLLM_BASE_URL" \
  "$PYTHON_BIN" "$ROOT_DIR/scripts/qwen35-chat-ui-server.py"
