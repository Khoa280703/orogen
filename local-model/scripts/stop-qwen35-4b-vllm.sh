#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export QWEN35_OUTPUT_DIR="${QWEN35_OUTPUT_DIR:-$ROOT_DIR/output/qwen35-4b-vllm}"
export QWEN35_PORT="${QWEN35_PORT:-8004}"

# shellcheck disable=SC1091
source "$ROOT_DIR/scripts/qwen35-service-common.sh"

stop_from_pid_file() {
  local pid_file="$1"
  local label="$2"
  local pid

  pid="$(read_pid "$pid_file")"
  if [[ -z "$pid" ]]; then
    return 0
  fi

  if ! is_pid_running "$pid"; then
    echo "==> ${label}: pidfile stale, xóa ${pid_file}"
    remove_pid_file "$pid_file"
    return 0
  fi

  echo "==> Dừng ${label} pid=${pid}"
  if ! terminate_pid "$pid" "$label" 30 TERM; then
    echo "==> Force kill ${label} pid=${pid}"
    kill -KILL "$pid" 2>/dev/null || true
  fi
  remove_pid_file "$pid_file"
}

stop_from_port() {
  local port="$1"
  local label="$2"
  mapfile -t pids < <(find_listening_pids "$port")

  if [[ "${#pids[@]}" -eq 0 ]]; then
    return 0
  fi

  echo "==> ${label}: port ${port} đang còn listener ngoài pidfile: ${pids[*]}"
  for pid in "${pids[@]}"; do
    if is_pid_running "$pid"; then
      if ! terminate_pid "$pid" "$label" 15 TERM; then
        kill -KILL "$pid" 2>/dev/null || true
      fi
    fi
  done
}

stop_from_pid_file "$HOST_PID_FILE" "Qwen 4B backend"
stop_from_port "$HOST_PORT" "Qwen 4B backend"

if port_is_listening "$HOST_PORT"; then
  echo "Backend vẫn còn listener trên port ${HOST_PORT}." >&2
  exit 1
fi

echo "==> Qwen 4B backend đã dừng."
