#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "==> Reset Qwen 4B backend"
"$ROOT_DIR/scripts/stop-qwen35-4b-vllm.sh"
"$ROOT_DIR/scripts/start-qwen35-4b-vllm.sh" "$@"
