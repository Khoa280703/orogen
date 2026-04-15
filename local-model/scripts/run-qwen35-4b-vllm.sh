#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MODEL_DIR="${QWEN35_MODEL_DIR:-$ROOT_DIR/models/qwen3.5-4b}"
VENV_DIR="${QWEN35_VENV_DIR:-$ROOT_DIR/.venv-qwen35-vllm}"
SERVED_MODEL_NAME="${QWEN35_SERVED_MODEL_NAME:-qwen3.5-4b}"
GPU_DEVICES="${QWEN35_GPU_DEVICES:-1}"
HOST_PORT="${QWEN35_PORT:-8004}"
HOST_BIND="${QWEN35_HOST:-0.0.0.0}"
MAX_MODEL_LEN="${QWEN35_MAX_MODEL_LEN:-131072}"
GPU_MEMORY_UTILIZATION="${QWEN35_GPU_MEMORY_UTILIZATION:-0.93}"
MAX_NUM_SEQS="${QWEN35_MAX_NUM_SEQS:-32}"
MAX_NUM_BATCHED_TOKENS="${QWEN35_MAX_NUM_BATCHED_TOKENS:-8192}"
BLOCK_SIZE="${QWEN35_BLOCK_SIZE:-32}"
SWAP_SPACE="${QWEN35_SWAP_SPACE:-0}"
TENSOR_PARALLEL_SIZE="${QWEN35_TENSOR_PARALLEL_SIZE:-1}"
CUDA_HOME="${QWEN35_CUDA_HOME:-}"
USE_SWAP_SPACE_FLAG="${QWEN35_USE_SWAP_SPACE_FLAG:-auto}"
FLASHINFER_WORKSPACE_BASE="${QWEN35_FLASHINFER_WORKSPACE_BASE:-$ROOT_DIR/.cache/flashinfer-qwen35-4b-vllm}"
REASONING_PARSER="${QWEN35_REASONING_PARSER:-qwen3}"
ENABLE_AUTO_TOOL_CHOICE="${QWEN35_ENABLE_AUTO_TOOL_CHOICE:-1}"
TOOL_CALL_PARSER="${QWEN35_TOOL_CALL_PARSER:-qwen3_xml}"
LANGUAGE_MODEL_ONLY="${QWEN35_LANGUAGE_MODEL_ONLY:-1}"

if [[ -z "$CUDA_HOME" ]]; then
  if command -v nvcc >/dev/null 2>&1; then
    CUDA_HOME="$(cd "$(dirname "$(command -v nvcc)")/.." && pwd)"
  else
    echo "Thiếu nvcc và chưa set QWEN35_CUDA_HOME." >&2
    exit 1
  fi
fi

if [[ ! -x "$VENV_DIR/bin/vllm" ]]; then
  echo "Chưa có vLLM trong $VENV_DIR. Chạy ./scripts/setup-qwen35-4b-vllm.sh trước." >&2
  exit 1
fi

if [[ ! -f "$MODEL_DIR/config.json" ]]; then
  echo "Chưa thấy model ở $MODEL_DIR. Chạy ./scripts/setup-qwen35-4b-vllm.sh trước." >&2
  exit 1
fi

if curl -fsS "http://127.0.0.1:${HOST_PORT}/health" >/dev/null 2>&1; then
  echo "Đã có service đang nghe ở port $HOST_PORT." >&2
  exit 1
fi

# shellcheck disable=SC1091
source "$VENV_DIR/bin/activate"

export CUDA_VISIBLE_DEVICES="$GPU_DEVICES"
export CUDACXX="${CUDACXX:-$CUDA_HOME/bin/nvcc}"
export PATH="$CUDA_HOME/bin:$PATH"
export LD_LIBRARY_PATH="$CUDA_HOME/lib64:${LD_LIBRARY_PATH:-}"
export HF_HUB_DISABLE_XET="${HF_HUB_DISABLE_XET:-1}"
export RAY_memory_monitor_refresh_ms="${RAY_memory_monitor_refresh_ms:-0}"
export NCCL_CUMEM_ENABLE="${NCCL_CUMEM_ENABLE:-0}"
export VLLM_ENABLE_CUDAGRAPH_GC="${VLLM_ENABLE_CUDAGRAPH_GC:-1}"
export VLLM_USE_FLASHINFER_SAMPLER="${VLLM_USE_FLASHINFER_SAMPLER:-1}"
export FLASHINFER_WORKSPACE_BASE

mkdir -p "$FLASHINFER_WORKSPACE_BASE"

supports_flag() {
  local flag="$1"
  vllm serve --help=all 2>/dev/null | grep -F -- "$flag" >/dev/null 2>&1
}

cmd=(
  vllm serve "$MODEL_DIR"
  --served-model-name "$SERVED_MODEL_NAME"
  --max-model-len "$MAX_MODEL_LEN"
  --max-num-seqs "$MAX_NUM_SEQS"
  --block-size "$BLOCK_SIZE"
  --max-num-batched-tokens "$MAX_NUM_BATCHED_TOKENS"
  --enable-prefix-caching
  --attention-backend FLASHINFER
  --tensor-parallel-size "$TENSOR_PARALLEL_SIZE"
  --gpu-memory-utilization "$GPU_MEMORY_UTILIZATION"
  --no-use-tqdm-on-load
  --host "$HOST_BIND"
  --port "$HOST_PORT"
)

if [[ "$LANGUAGE_MODEL_ONLY" == "1" || "$LANGUAGE_MODEL_ONLY" == "true" ]]; then
  if supports_flag "--language-model-only"; then
    cmd+=(--language-model-only)
  fi
fi

if [[ -n "$REASONING_PARSER" ]] && supports_flag "--reasoning-parser"; then
  cmd+=(--reasoning-parser "$REASONING_PARSER")
fi

if [[ "$ENABLE_AUTO_TOOL_CHOICE" == "1" || "$ENABLE_AUTO_TOOL_CHOICE" == "true" ]]; then
  if supports_flag "--enable-auto-tool-choice"; then
    cmd+=(--enable-auto-tool-choice)
  fi
  if [[ -n "$TOOL_CALL_PARSER" ]] && supports_flag "--tool-call-parser"; then
    cmd+=(--tool-call-parser "$TOOL_CALL_PARSER")
  fi
fi

if [[ "$USE_SWAP_SPACE_FLAG" == "1" || "$USE_SWAP_SPACE_FLAG" == "true" ]]; then
  cmd+=(--swap-space "$SWAP_SPACE")
elif [[ "$USE_SWAP_SPACE_FLAG" == "auto" ]] && supports_flag "--swap-space"; then
  cmd+=(--swap-space "$SWAP_SPACE")
fi

cmd+=("$@")

echo "==> Starting host-side vLLM from $VENV_DIR"
echo "==> CUDA_VISIBLE_DEVICES=$CUDA_VISIBLE_DEVICES"
echo "==> model=$MODEL_DIR"
echo "==> port=$HOST_PORT max_model_len=$MAX_MODEL_LEN max_num_seqs=$MAX_NUM_SEQS"
echo "==> tensor_parallel_size=$TENSOR_PARALLEL_SIZE language_model_only=$LANGUAGE_MODEL_ONLY"
echo "==> reasoning_parser=${REASONING_PARSER:-<none>} auto_tool_choice=${ENABLE_AUTO_TOOL_CHOICE} tool_call_parser=${TOOL_CALL_PARSER:-<none>}"

exec "${cmd[@]}"
