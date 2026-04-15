#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export QWEN35_OUTPUT_DIR="${QWEN35_OUTPUT_DIR:-$ROOT_DIR/output/qwen35-4b-gateway}"
export QWEN35_PORT="${QWEN35_PORT:-8004}"

# shellcheck disable=SC1091
source "$ROOT_DIR/scripts/qwen35-service-common.sh"

pid="$(read_pid "$HOST_PID_FILE")"
if [[ -n "$pid" ]] && is_pid_running "$pid"; then
  echo "==> Dừng Qwen 4B gateway pid=${pid}"
  if ! terminate_pid "$pid" "Qwen 4B gateway" 15 TERM; then
    kill -KILL "$pid" 2>/dev/null || true
  fi
fi
remove_pid_file "$HOST_PID_FILE"

if port_is_listening "$HOST_PORT"; then
  mapfile -t pids < <(find_listening_pids "$HOST_PORT")
  for listener_pid in "${pids[@]}"; do
    kill -TERM "$listener_pid" 2>/dev/null || true
    sleep 1
    kill -KILL "$listener_pid" 2>/dev/null || true
  done
fi

echo "==> Qwen 4B gateway đã dừng."
