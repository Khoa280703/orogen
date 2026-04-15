#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
UI_PORT="${QWEN35_UI_PORT:-8010}"
UI_OUTPUT_DIR="${QWEN35_UI_OUTPUT_DIR:-$ROOT_DIR/output/qwen35-4b-cluster-ui}"
UI_STATE_DIR="${QWEN35_UI_STATE_DIR:-$UI_OUTPUT_DIR/runtime}"
UI_PID_FILE="${QWEN35_UI_PID_FILE:-$UI_STATE_DIR/ui-${UI_PORT}.pid}"

stop_instance() {
  local port="$1"
  local output_dir="$2"

  QWEN35_PORT="$port" \
  QWEN35_OUTPUT_DIR="$output_dir" \
  "$ROOT_DIR/scripts/stop-qwen35-4b-vllm.sh"
}

stop_ui_pid() {
  local pid=""
  if [[ -f "$UI_PID_FILE" ]]; then
    pid="$(tr -d '[:space:]' <"$UI_PID_FILE")"
  fi

  if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
    echo "==> Dừng Qwen 4B UI pid=${pid}"
    kill -TERM "$pid" 2>/dev/null || true
    sleep 2
    kill -KILL "$pid" 2>/dev/null || true
  fi

  rm -f "$UI_PID_FILE"

  if command -v lsof >/dev/null 2>&1; then
    while read -r ui_pid; do
      [[ -z "$ui_pid" ]] && continue
      kill -TERM "$ui_pid" 2>/dev/null || true
      sleep 1
      kill -KILL "$ui_pid" 2>/dev/null || true
    done < <(lsof -tiTCP:"$UI_PORT" -sTCP:LISTEN 2>/dev/null | sort -u)
  fi
}

stop_ui_pid
"$ROOT_DIR/scripts/stop-qwen35-4b-gateway.sh"
stop_instance "${QWEN35_PORT_GPU0:-8100}" "${QWEN35_OUTPUT_DIR_GPU0:-$ROOT_DIR/output/qwen35-4b-gpu0}"
stop_instance "${QWEN35_PORT_GPU1:-8101}" "${QWEN35_OUTPUT_DIR_GPU1:-$ROOT_DIR/output/qwen35-4b-gpu1}"
stop_instance "${QWEN35_PORT_GPU2:-8102}" "${QWEN35_OUTPUT_DIR_GPU2:-$ROOT_DIR/output/qwen35-4b-gpu2}"

echo "==> Qwen 4B cluster đã dừng."
