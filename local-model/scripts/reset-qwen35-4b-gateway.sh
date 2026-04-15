#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "==> Reset Qwen 4B gateway"
"$ROOT_DIR/scripts/stop-qwen35-4b-gateway.sh"
"$ROOT_DIR/scripts/start-qwen35-4b-gateway.sh" "$@"
