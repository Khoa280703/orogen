#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

export QWEN35_MODEL_ID="${QWEN35_MODEL_ID:-Qwen/Qwen3.5-4B}"
export QWEN35_MODEL_DIR="${QWEN35_MODEL_DIR:-$ROOT_DIR/models/qwen3.5-4b}"
export QWEN35_OUTPUT_DIR="${QWEN35_OUTPUT_DIR:-$ROOT_DIR/output/qwen35-4b-vllm}"
export QWEN35_VENV_DIR="${QWEN35_VENV_DIR:-$ROOT_DIR/.venv-qwen35-vllm}"

if [[ -x "$QWEN35_VENV_DIR/bin/vllm" ]]; then
  export HF_HUB_DISABLE_XET="${HF_HUB_DISABLE_XET:-1}"
  mkdir -p "$QWEN35_MODEL_DIR"
  echo "==> Reuse vLLM có sẵn ở $QWEN35_VENV_DIR"
  echo "==> Download model: $QWEN35_MODEL_ID"
  exec hf download "$QWEN35_MODEL_ID" --local-dir "$QWEN35_MODEL_DIR"
fi

exec "$ROOT_DIR/scripts/setup-qwen35-vllm-runtime.sh"
