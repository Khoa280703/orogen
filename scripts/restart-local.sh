#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WEB_DIR="$ROOT_DIR/web"
LOG_DIR="$ROOT_DIR/tmp/local-runtime"
BACKEND_LOG="$LOG_DIR/backend.log"
FRONTEND_LOG="$LOG_DIR/frontend.log"
BACKEND_PID_FILE="$LOG_DIR/backend.pid"
FRONTEND_PID_FILE="$LOG_DIR/frontend.pid"

mkdir -p "$LOG_DIR"

kill_matching_processes() {
  local pattern="$1"
  mapfile -t pids < <(pgrep -f "$pattern" || true)
  if ((${#pids[@]} == 0)); then
    return
  fi

  echo "Killing: $pattern -> ${pids[*]}"
  kill "${pids[@]}" 2>/dev/null || true

  sleep 1

  mapfile -t stubborn_pids < <(pgrep -f "$pattern" || true)
  if ((${#stubborn_pids[@]} > 0)); then
    echo "Force killing: $pattern -> ${stubborn_pids[*]}"
    kill -9 "${stubborn_pids[@]}" 2>/dev/null || true
  fi
}

wait_for_http() {
  local url="$1"
  local name="$2"

  for _ in $(seq 1 60); do
    if curl -fsS "$url" >/dev/null 2>&1; then
      echo "$name is ready: $url"
      return 0
    fi
    sleep 1
  done

  echo "$name failed to become ready: $url" >&2
  return 1
}

echo "Resetting local backend + frontend..."

kill_matching_processes 'target/debug/grok-local serve'
kill_matching_processes 'cargo run.*grok-local.*serve'
kill_matching_processes 'next dev --webpack -p 5169'

if [[ -x "$ROOT_DIR/target/debug/grok-local" ]]; then
  BACKEND_CMD=(bash -lc "cd '$ROOT_DIR' && exec target/debug/grok-local serve")
else
  BACKEND_CMD=(bash -lc "cd '$ROOT_DIR' && exec cargo run --bin grok-local -- serve")
fi

nohup "${BACKEND_CMD[@]}" >"$BACKEND_LOG" 2>&1 </dev/null &
BACKEND_PID=$!
echo "$BACKEND_PID" >"$BACKEND_PID_FILE"
echo "Started backend pid=$BACKEND_PID"

nohup bash -lc "cd '$WEB_DIR' && exec npm run dev" >"$FRONTEND_LOG" 2>&1 </dev/null &
FRONTEND_PID=$!
echo "$FRONTEND_PID" >"$FRONTEND_PID_FILE"
echo "Started frontend pid=$FRONTEND_PID"

wait_for_http "http://127.0.0.1:3069/" "Backend"
wait_for_http "http://127.0.0.1:5169/" "Frontend"

echo "Done."
echo "Backend log: $BACKEND_LOG"
echo "Frontend log: $FRONTEND_LOG"
