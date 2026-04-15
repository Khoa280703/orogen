#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
UI_PORT="${QWEN35_UI_PORT:-8010}"
UI_HOST="${QWEN35_UI_HOST:-0.0.0.0}"
UI_BASE_URL="${QWEN35_UI_VLLM_BASE_URL:-http://127.0.0.1:8004}"
UI_DEFAULT_MODEL="${QWEN35_UI_DEFAULT_MODEL:-qwen3.5-4b}"
UI_OUTPUT_DIR="${QWEN35_UI_OUTPUT_DIR:-$ROOT_DIR/output/qwen35-4b-cluster-ui}"
UI_STATE_DIR="${QWEN35_UI_STATE_DIR:-$UI_OUTPUT_DIR/runtime}"
UI_LOG="${QWEN35_UI_LOG:-$UI_OUTPUT_DIR/ui-${UI_PORT}.log}"
UI_PID_FILE="${QWEN35_UI_PID_FILE:-$UI_STATE_DIR/ui-${UI_PORT}.pid}"

start_instance() {
  local gpu="$1"
  local port="$2"
  local output_dir="$3"
  local gpu_memory_utilization_var="QWEN35_GPU_MEMORY_UTILIZATION_GPU${gpu}"
  local gpu_memory_utilization="${!gpu_memory_utilization_var:-}"

  if [[ -z "$gpu_memory_utilization" ]]; then
    if [[ "$gpu" == "0" ]]; then
      gpu_memory_utilization="${QWEN35_GPU_MEMORY_UTILIZATION_GPU0_DEFAULT:-0.88}"
    else
      gpu_memory_utilization="${QWEN35_GPU_MEMORY_UTILIZATION:-0.93}"
    fi
  fi

  echo "==> Start Qwen 4B replica gpu=${gpu} port=${port} gpu_memory_utilization=${gpu_memory_utilization}"
  QWEN35_GPU_DEVICES="$gpu" \
  QWEN35_PORT="$port" \
  QWEN35_OUTPUT_DIR="$output_dir" \
  QWEN35_GPU_MEMORY_UTILIZATION="$gpu_memory_utilization" \
  "$ROOT_DIR/scripts/start-qwen35-4b-vllm.sh"
}

start_instance 0 "${QWEN35_PORT_GPU0:-8100}" "${QWEN35_OUTPUT_DIR_GPU0:-$ROOT_DIR/output/qwen35-4b-gpu0}"
start_instance 1 "${QWEN35_PORT_GPU1:-8101}" "${QWEN35_OUTPUT_DIR_GPU1:-$ROOT_DIR/output/qwen35-4b-gpu1}"
start_instance 2 "${QWEN35_PORT_GPU2:-8102}" "${QWEN35_OUTPUT_DIR_GPU2:-$ROOT_DIR/output/qwen35-4b-gpu2}"
"$ROOT_DIR/scripts/start-qwen35-4b-gateway.sh"

mkdir -p "$UI_OUTPUT_DIR" "$UI_STATE_DIR"

if ss -ltnH | awk -v target=":${UI_PORT}" '$4 ~ target"$" {found=1} END {exit found ? 0 : 1}'; then
  echo "Port UI ${UI_PORT} đang bị process khác chiếm. Bỏ qua start UI." >&2
  exit 0
fi

echo "==> Start Qwen 4B UI trên port ${UI_PORT}, target=${UI_BASE_URL}"
: >"$UI_LOG"
(
  cd "$ROOT_DIR"
  setsid env \
    QWEN35_UI_HOST="$UI_HOST" \
    QWEN35_UI_PORT="$UI_PORT" \
    QWEN35_UI_VLLM_BASE_URL="$UI_BASE_URL" \
    QWEN35_UI_DEFAULT_MODEL="$UI_DEFAULT_MODEL" \
    "$ROOT_DIR/scripts/run-qwen35-chat-ui.sh" >>"$UI_LOG" 2>&1 < /dev/null &
  echo $! >"$UI_PID_FILE"
)

echo "==> UI pid=$(tr -d '[:space:]' <"$UI_PID_FILE"), log=${UI_LOG}"
echo "==> 4B cluster đã được yêu cầu khởi động."
