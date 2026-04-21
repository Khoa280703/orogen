#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VENV_DIR="${QWEN35_SFT_VENV_DIR:-$ROOT_DIR/.venv-qwen35-sft}"
PYTHON_BIN="${QWEN35_SFT_PYTHON_BIN:-python3}"
TORCH_INDEX_URL="${QWEN35_SFT_TORCH_INDEX_URL:-https://download.pytorch.org/whl/cu128}"
TORCH_PACKAGE="${QWEN35_SFT_TORCH_PACKAGE:-torch==2.10.0}"
EXTRA_TORCH_PACKAGES="${QWEN35_SFT_EXTRA_TORCH_PACKAGES:-}"
DATASETS_PACKAGE="${QWEN35_SFT_DATASETS_PACKAGE:-datasets==4.8.4}"
TRANSFORMERS_PACKAGE="${QWEN35_SFT_TRANSFORMERS_PACKAGE:-transformers==5.5.4}"
PEFT_PACKAGE="${QWEN35_SFT_PEFT_PACKAGE:-peft==0.19.0}"
TRL_PACKAGE="${QWEN35_SFT_TRL_PACKAGE:-trl==1.1.0}"
ACCELERATE_PACKAGE="${QWEN35_SFT_ACCELERATE_PACKAGE:-accelerate==1.13.0}"
BITSANDBYTES_PACKAGE="${QWEN35_SFT_BITSANDBYTES_PACKAGE:-bitsandbytes==0.49.2}"
SFT_PACKAGES=(
  "$DATASETS_PACKAGE"
  "$TRANSFORMERS_PACKAGE"
  "$PEFT_PACKAGE"
  "$TRL_PACKAGE"
  "$ACCELERATE_PACKAGE"
  "$BITSANDBYTES_PACKAGE"
)

if ! command -v "$PYTHON_BIN" >/dev/null 2>&1; then
  echo "Không tìm thấy Python: $PYTHON_BIN" >&2
  exit 1
fi

if [[ ! -x "$VENV_DIR/bin/python" ]]; then
  echo "==> Create SFT venv: $VENV_DIR"
  "$PYTHON_BIN" -m venv "$VENV_DIR"
fi

# shellcheck disable=SC1091
source "$VENV_DIR/bin/activate"

echo "==> Upgrade pip build tooling"
python -m pip install --upgrade pip setuptools wheel

echo "==> Install torch runtime"
python -m pip install --upgrade --index-url "$TORCH_INDEX_URL" "$TORCH_PACKAGE"
if [[ -n "$EXTRA_TORCH_PACKAGES" ]]; then
  # shellcheck disable=SC2206
  extra_packages=( $EXTRA_TORCH_PACKAGES )
  python -m pip install --upgrade --index-url "$TORCH_INDEX_URL" "${extra_packages[@]}"
fi

echo "==> Install SFT packages"
python -m pip install --upgrade "${SFT_PACKAGES[@]}"

echo "==> Runtime check"
python "$ROOT_DIR/scripts/check-qwen35-workflow-trace-sft-runtime.py" \
  --output "$ROOT_DIR/output/qwen35-workflow-trace-sft-runtime-check.json"

echo "==> Done"
echo "Venv: $VENV_DIR"
