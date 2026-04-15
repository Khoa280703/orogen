#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export QWEN35_OUTPUT_DIR="${QWEN35_OUTPUT_DIR:-$ROOT_DIR/output/qwen35-4b-vllm}"
export QWEN35_PORT="${QWEN35_PORT:-8004}"

# shellcheck disable=SC1091
source "$ROOT_DIR/scripts/qwen35-service-common.sh"

BACKEND_WAIT_SECONDS="${QWEN35_START_BACKEND_WAIT_SECONDS:-600}"
BACKEND_HEALTH_URL="${QWEN35_BACKEND_HEALTH_URL:-http://127.0.0.1:${HOST_PORT}/health}"

backend_pid="$(read_pid "$HOST_PID_FILE")"
if is_pid_running "$backend_pid"; then
  echo "Qwen 4B backend đã chạy với pid=${backend_pid} trên port ${HOST_PORT}."
  exit 0
fi

if port_is_listening "$HOST_PORT"; then
  echo "Port ${HOST_PORT} đang bị process khác chiếm. Dừng process đó trước khi start." >&2
  exit 1
fi

remove_pid_file "$HOST_PID_FILE"
echo "==> Start Qwen 4B backend trên port ${HOST_PORT}"
start_detached_logged "$HOST_PID_FILE" "$HOST_LOG" "$ROOT_DIR/scripts/run-qwen35-4b-vllm.sh" "$@"
backend_pid="$(read_pid "$HOST_PID_FILE")"
echo "==> Backend pid=${backend_pid}, log=${HOST_LOG}"

if ! wait_for_http_with_pid "$BACKEND_HEALTH_URL" "$BACKEND_WAIT_SECONDS" "Qwen 4B backend" "$backend_pid"; then
  echo "Backend không lên kịp. Kiểm tra log: ${HOST_LOG}" >&2
  exit 1
fi

echo
echo "Backend : ${BACKEND_HEALTH_URL}"
echo "Host log: ${HOST_LOG}"
