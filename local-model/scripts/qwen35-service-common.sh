#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT_DIR="${QWEN35_OUTPUT_DIR:-$ROOT_DIR/output/qwen35-27b-vllm}"
STATE_DIR="${QWEN35_STATE_DIR:-$OUTPUT_DIR/runtime}"
HOST_PORT="${QWEN35_PORT:-8002}"
UI_PORT="${QWEN35_UI_PORT:-8010}"
HOST_LOG="${QWEN35_HOST_LOG:-$OUTPUT_DIR/host-${HOST_PORT}.log}"
UI_LOG="${QWEN35_UI_LOG:-$OUTPUT_DIR/ui-${UI_PORT}.log}"
HOST_PID_FILE="${QWEN35_HOST_PID_FILE:-$STATE_DIR/host-${HOST_PORT}.pid}"
UI_PID_FILE="${QWEN35_UI_PID_FILE:-$STATE_DIR/ui-${UI_PORT}.pid}"
FLASHINFER_WORKSPACE_BASE="${QWEN35_FLASHINFER_WORKSPACE_BASE:-$ROOT_DIR/.cache/flashinfer-qwen35-vllm}"

mkdir -p "$OUTPUT_DIR" "$STATE_DIR"

read_pid() {
  local pid_file="$1"
  if [[ -f "$pid_file" ]]; then
    tr -d '[:space:]' <"$pid_file"
  fi
}

remove_pid_file() {
  local pid_file="$1"
  rm -f "$pid_file"
}

is_pid_running() {
  local pid="${1:-}"
  [[ -n "$pid" ]] && [[ "$pid" =~ ^[0-9]+$ ]] && kill -0 "$pid" 2>/dev/null
}

port_is_listening() {
  local port="$1"
  ss -ltnH | awk -v target=":$port" '$4 ~ target"$" {found=1} END {exit found ? 0 : 1}'
}

find_listening_pids() {
  local port="$1"

  if command -v lsof >/dev/null 2>&1; then
    lsof -tiTCP:"$port" -sTCP:LISTEN 2>/dev/null | sort -u
    return 0
  fi

  ss -ltnpH | awk -v target=":$port" '
    $4 ~ target"$" {
      while (match($0, /pid=[0-9]+/)) {
        pid = substr($0, RSTART + 4, RLENGTH - 4)
        print pid
        $0 = substr($0, RSTART + RLENGTH)
      }
    }
  ' | sort -u
}

wait_for_http() {
  local url="$1"
  local timeout_seconds="$2"
  local label="$3"
  local started_at
  started_at="$(date +%s)"

  while true; do
    if curl -fsS "$url" >/dev/null 2>&1; then
      return 0
    fi

    if (( "$(date +%s)" - started_at >= timeout_seconds )); then
      echo "Hết thời gian chờ ${label}: ${url}" >&2
      return 1
    fi

    sleep 2
  done
}

wait_for_http_with_pid() {
  local url="$1"
  local timeout_seconds="$2"
  local label="$3"
  local pid="$4"
  local started_at
  started_at="$(date +%s)"

  while true; do
    if curl -fsS "$url" >/dev/null 2>&1; then
      return 0
    fi

    if ! is_pid_running "$pid"; then
      echo "${label} đã thoát trước khi health check pass." >&2
      return 1
    fi

    if (( "$(date +%s)" - started_at >= timeout_seconds )); then
      echo "Hết thời gian chờ ${label}: ${url}" >&2
      return 1
    fi

    sleep 2
  done
}

terminate_pid() {
  local pid="$1"
  local label="$2"
  local timeout_seconds="${3:-30}"
  local signal="${4:-TERM}"

  if ! is_pid_running "$pid"; then
    return 0
  fi

  kill "-$signal" "$pid" 2>/dev/null || true

  local started_at
  started_at="$(date +%s)"
  while is_pid_running "$pid"; do
    if (( "$(date +%s)" - started_at >= timeout_seconds )); then
      echo "${label} vẫn chưa dừng sau ${timeout_seconds}s." >&2
      return 1
    fi
    sleep 1
  done

  return 0
}

start_detached_logged() {
  local pid_file="$1"
  local log_file="$2"
  shift 2

  : >"$log_file"
  (
    cd "$ROOT_DIR"
    setsid "$@" >>"$log_file" 2>&1 < /dev/null &
    echo $! >"$pid_file"
  )
}
