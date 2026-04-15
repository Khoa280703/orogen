#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "==> Reset Qwen 4B cluster"
"$ROOT_DIR/scripts/stop-qwen35-4b-cluster.sh"
"$ROOT_DIR/scripts/start-qwen35-4b-cluster.sh" "$@"
