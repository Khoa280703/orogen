#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export QWEN35_OUTPUT_DIR="${QWEN35_OUTPUT_DIR:-$ROOT_DIR/output/qwen35-4b-gateway}"
export QWEN35_PORT="${QWEN35_PORT:-8004}"
export QWEN35_GATEWAY_PORT="${QWEN35_GATEWAY_PORT:-$QWEN35_PORT}"

# shellcheck disable=SC1091
source "$ROOT_DIR/scripts/qwen35-service-common.sh"

BACKEND_WAIT_SECONDS="${QWEN35_START_BACKEND_WAIT_SECONDS:-60}"
BACKEND_HEALTH_URL="${QWEN35_BACKEND_HEALTH_URL:-http://127.0.0.1:${HOST_PORT}/health}"

backend_pid="$(read_pid "$HOST_PID_FILE")"
if is_pid_running "$backend_pid"; then
  echo "Qwen 4B gateway đã chạy với pid=${backend_pid} trên port ${HOST_PORT}."
  exit 0
fi

if port_is_listening "$HOST_PORT"; then
  echo "Port ${HOST_PORT} đang bị process khác chiếm. Dừng process đó trước khi start." >&2
  exit 1
fi

remove_pid_file "$HOST_PID_FILE"
echo "==> Start Qwen 4B gateway trên port ${HOST_PORT}"
start_detached_logged "$HOST_PID_FILE" "$HOST_LOG" python3 "$ROOT_DIR/scripts/qwen35-4b-gateway.py"
backend_pid="$(read_pid "$HOST_PID_FILE")"
echo "==> Gateway pid=${backend_pid}, log=${HOST_LOG}"

if ! wait_for_http_with_pid "$BACKEND_HEALTH_URL" "$BACKEND_WAIT_SECONDS" "Qwen 4B gateway" "$backend_pid"; then
  echo "Gateway không lên kịp. Kiểm tra log: ${HOST_LOG}" >&2
  exit 1
fi

echo
echo "Gateway : ${BACKEND_HEALTH_URL}"
echo "Host log: ${HOST_LOG}"
