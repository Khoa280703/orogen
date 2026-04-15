#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MODEL_ID="${QWEN35_MODEL_ID:-cyankiwi/Qwen3.5-27B-AWQ-BF16-INT4}"
MODEL_DIR="${QWEN35_MODEL_DIR:-$ROOT_DIR/models/qwen3.5-27b-awq-bf16-int4}"
HF_CACHE_DIR="${HF_HOME:-$HOME/.cache/huggingface}"
OUTPUT_DIR="${QWEN35_OUTPUT_DIR:-$ROOT_DIR/output/qwen35-27b-vllm}"
VENV_DIR="${QWEN35_VENV_DIR:-$ROOT_DIR/.venv-qwen35-vllm}"
VLLM_SRC_DIR="${QWEN35_VLLM_SRC_DIR:-$HOME/.cache/vllm-src}"
VLLM_REF="${QWEN35_VLLM_REF:-main}"
VLLM_INSTALL_MODE="${QWEN35_VLLM_INSTALL_MODE:-auto}"
VLLM_WHEEL_VERSION="${QWEN35_VLLM_WHEEL_VERSION:-0.19.0}"
PYTHON_BIN="${QWEN35_PYTHON_BIN:-python3}"
CUDA_HOME="${QWEN35_CUDA_HOME:-}"

apply_cuda12_cumem_compat_patch() {
  local compat_file="$VLLM_SRC_DIR/csrc/cumem_allocator.cpp"

  if [[ ! -f "$compat_file" ]]; then
    return 0
  fi

  if grep -q 'defined(CU_DEVICE_ATTRIBUTE_HANDLE_TYPE_FABRIC_SUPPORTED)' "$compat_file"; then
    return 0
  fi

  COMPAT_FILE="$compat_file" "$PYTHON_BIN" - <<'PY'
from pathlib import Path
import os
import sys

path = Path(os.environ["COMPAT_FILE"])
text = path.read_text()

old_probe = """  int fab_flag = 0;
  CUresult fab_result = cuDeviceGetAttribute(
      &fab_flag, CU_DEVICE_ATTRIBUTE_HANDLE_TYPE_FABRIC_SUPPORTED, device);
  if (fab_result == CUDA_SUCCESS &&
      fab_flag) {  // support fabric handle if possible
    prop.requestedHandleTypes = CU_MEM_HANDLE_TYPE_FABRIC;
  }
"""

new_probe = """  int fab_flag = 0;
#if defined(CU_DEVICE_ATTRIBUTE_HANDLE_TYPE_FABRIC_SUPPORTED) && \\
    defined(CU_MEM_HANDLE_TYPE_FABRIC)
  CUresult fab_result = cuDeviceGetAttribute(
      &fab_flag, CU_DEVICE_ATTRIBUTE_HANDLE_TYPE_FABRIC_SUPPORTED, device);
  if (fab_result == CUDA_SUCCESS &&
      fab_flag) {  // support fabric handle if possible
    prop.requestedHandleTypes = CU_MEM_HANDLE_TYPE_FABRIC;
  }
#endif
"""

old_fallback = """  if (ret) {
    if (fab_flag &&
        (ret == CUDA_ERROR_NOT_PERMITTED || ret == CUDA_ERROR_NOT_SUPPORTED)) {
      // Fabric allocation may fail without multi-node nvlink,
      // fallback to POSIX file descriptor
      prop.requestedHandleTypes = CU_MEM_HANDLE_TYPE_POSIX_FILE_DESCRIPTOR;
      CUDA_CHECK(cuMemCreate(p_memHandle, size, &prop, 0));
    } else {
      CUDA_CHECK(ret);
    }
  }
"""

new_fallback = """  if (ret) {
#if defined(CU_DEVICE_ATTRIBUTE_HANDLE_TYPE_FABRIC_SUPPORTED) && \\
    defined(CU_MEM_HANDLE_TYPE_FABRIC)
    if (fab_flag &&
        (ret == CUDA_ERROR_NOT_PERMITTED || ret == CUDA_ERROR_NOT_SUPPORTED)) {
      // Fabric allocation may fail without multi-node nvlink,
      // fallback to POSIX file descriptor
      prop.requestedHandleTypes = CU_MEM_HANDLE_TYPE_POSIX_FILE_DESCRIPTOR;
      CUDA_CHECK(cuMemCreate(p_memHandle, size, &prop, 0));
    } else {
      CUDA_CHECK(ret);
    }
#else
    CUDA_CHECK(ret);
#endif
  }
"""

updated = text.replace(old_probe, new_probe).replace(old_fallback, new_fallback)
if updated == text:
    print("Không áp được patch CUDA 12 compat vào cumem_allocator.cpp", file=sys.stderr)
    sys.exit(1)

path.write_text(updated)
PY
}

apply_cuda12_activation_kernel_compat_patch() {
  local compat_file="$VLLM_SRC_DIR/csrc/quantization/activation_kernels.cu"

  if [[ ! -f "$compat_file" ]]; then
    return 0
  fi

  if grep -q 'VLLM_USE_DEEP_GEMM_FP8_ACTIVATION_KERNELS' "$compat_file"; then
    return 0
  fi

  COMPAT_FILE="$compat_file" "$PYTHON_BIN" - <<'PY'
from pathlib import Path
import os
import sys

path = Path(os.environ["COMPAT_FILE"])
text = path.read_text()

old_header = '#include "core/registration.h"\nnamespace vllm {\n'
new_header = '''#include "core/registration.h"

#if !defined(USE_ROCM) && defined(__CUDACC_VER_MAJOR__) && \\
    ((__CUDACC_VER_MAJOR__ > 12) || \\
     (__CUDACC_VER_MAJOR__ == 12 && __CUDACC_VER_MINOR__ >= 8))
  #define VLLM_USE_DEEP_GEMM_FP8_ACTIVATION_KERNELS 1
#else
  #define VLLM_USE_DEEP_GEMM_FP8_ACTIVATION_KERNELS 0
#endif

namespace vllm {
'''

old_kernel_block = '''// We use the following values for fp8 min/max:
//  __nv_fp8_e4m3 = (-448, +448)
//  __nv_fp8_e4m3uz = (-240.0, +240.0)
// It is currently assumed that only
'''
new_kernel_block = '''#if VLLM_USE_DEEP_GEMM_FP8_ACTIVATION_KERNELS

// We use the following values for fp8 min/max:
//  __nv_fp8_e4m3 = (-448, +448)
//  __nv_fp8_e4m3uz = (-240.0, +240.0)
// It is currently assumed that only
'''

old_namespace_close = '\n}  // namespace vllm\n'
new_namespace_close = '\n#endif\n}\n\n}  // namespace vllm\n'

old_fn_guard = '''void persistent_masked_m_silu_mul_quant(
    const at::Tensor& input,              // (E, T, 2*H)
    const at::Tensor& tokens_per_expert,  // (E)
    at::Tensor& y_q,                      // (E, T, H) [OUT]
    at::Tensor& y_s,                      // (E, T, H//group_size) [OUT]
    bool cast_scale_ue8m0) {
#ifndef USE_ROCM
'''

new_fn_guard = '''void persistent_masked_m_silu_mul_quant(
    const at::Tensor& input,              // (E, T, 2*H)
    const at::Tensor& tokens_per_expert,  // (E)
    at::Tensor& y_q,                      // (E, T, H) [OUT]
    at::Tensor& y_s,                      // (E, T, H//group_size) [OUT]
    bool cast_scale_ue8m0) {
#if VLLM_USE_DEEP_GEMM_FP8_ACTIVATION_KERNELS
'''

old_fn_tail = '''  LAUNCH_ON_H(uint8_t, stride_ys_e, stride_ys_t, stride_ys_g, stride_ys_p,
              true);

#endif
}
'''

new_fn_tail = '''  LAUNCH_ON_H(uint8_t, stride_ys_e, stride_ys_t, stride_ys_g, stride_ys_p,
              true);

#else
  TORCH_CHECK(
      false,
      "persistent_masked_m_silu_mul_quant requires CUDA 12.8+ when building "
      "vLLM from source on this machine.");
#endif
}
'''

updated = text
for old, new in [
    (old_header, new_header),
    (old_kernel_block, new_kernel_block),
    (old_namespace_close, new_namespace_close),
    (old_fn_guard, new_fn_guard),
    (old_fn_tail, new_fn_tail),
]:
    updated = updated.replace(old, new, 1)

if updated == text:
    print("Không áp được patch CUDA 12 activation compat", file=sys.stderr)
    sys.exit(1)

path.write_text(updated)
PY
}

if ! command -v hf >/dev/null 2>&1; then
  echo "Thiếu hf CLI. Cài huggingface_hub trước." >&2
  exit 1
fi

if ! command -v git >/dev/null 2>&1; then
  echo "Thiếu git." >&2
  exit 1
fi

if ! command -v "$PYTHON_BIN" >/dev/null 2>&1; then
  echo "Không tìm thấy Python: $PYTHON_BIN" >&2
  exit 1
fi

if ! command -v nvcc >/dev/null 2>&1 && [[ -z "$CUDA_HOME" ]]; then
  echo "Thiếu nvcc và chưa set QWEN35_CUDA_HOME." >&2
  exit 1
fi

if [[ -z "$CUDA_HOME" ]]; then
  CUDA_BIN_DIR="$(dirname "$(command -v nvcc)")"
  CUDA_HOME="$(cd "$CUDA_BIN_DIR/.." && pwd)"
fi

mkdir -p "$MODEL_DIR" "$HF_CACHE_DIR" "$OUTPUT_DIR" "$(dirname "$VLLM_SRC_DIR")"

export HF_HUB_DISABLE_XET="${HF_HUB_DISABLE_XET:-1}"
export CUDACXX="${CUDACXX:-$CUDA_HOME/bin/nvcc}"
export MAX_JOBS="${MAX_JOBS:-1}"
export PATH="$CUDA_HOME/bin:$PATH"
export LD_LIBRARY_PATH="$CUDA_HOME/lib64:${LD_LIBRARY_PATH:-}"

if [[ ! -d "$VLLM_SRC_DIR/.git" ]]; then
  echo "==> Clone vLLM source: $VLLM_SRC_DIR"
  git clone --depth 1 --branch "$VLLM_REF" https://github.com/vllm-project/vllm.git "$VLLM_SRC_DIR"
else
  echo "==> Update vLLM source: $VLLM_SRC_DIR"
  git -C "$VLLM_SRC_DIR" fetch --depth 1 origin "$VLLM_REF"
  git -C "$VLLM_SRC_DIR" checkout -B "local-build-$VLLM_REF" FETCH_HEAD
fi

echo "==> Apply CUDA 12 cumem compatibility patch if needed"
apply_cuda12_cumem_compat_patch
echo "==> Apply CUDA 12 activation-kernel compatibility patch if needed"
apply_cuda12_activation_kernel_compat_patch

if [[ ! -x "$VENV_DIR/bin/python" ]]; then
  echo "==> Create venv: $VENV_DIR"
  "$PYTHON_BIN" -m venv "$VENV_DIR"
fi

# shellcheck disable=SC1091
source "$VENV_DIR/bin/activate"
python -m pip install -U pip 'setuptools<81.0.0' wheel packaging ninja cmake

echo "==> Download model: $MODEL_ID"
hf download "$MODEL_ID" --local-dir "$MODEL_DIR"

if [[ -f "$VLLM_SRC_DIR/requirements/build.txt" ]]; then
  python -m pip install -r "$VLLM_SRC_DIR/requirements/build.txt"
fi

echo "==> Build vLLM from source"
build_from_source() {
  (
    cd "$VLLM_SRC_DIR"
    python -m pip install -e .
  )
}

install_from_wheel() {
  echo "==> Install vLLM wheel: ${VLLM_WHEEL_VERSION}"
  python -m pip install "vllm==${VLLM_WHEEL_VERSION}"
}

case "$VLLM_INSTALL_MODE" in
  source)
    build_from_source
    ;;
  wheel)
    install_from_wheel
    ;;
  auto)
    if ! build_from_source; then
      echo "==> Source build fail, fallback sang wheel ${VLLM_WHEEL_VERSION}" >&2
      install_from_wheel
    fi
    ;;
  *)
    echo "QWEN35_VLLM_INSTALL_MODE không hợp lệ: $VLLM_INSTALL_MODE" >&2
    exit 1
    ;;
esac

echo "==> Setup xong"
printf 'venv=%s\nvllm_src=%s\ncuda_home=%s\nmodel_dir=%s\nhf_cache=%s\noutput_dir=%s\ninstall_mode=%s\nwheel_version=%s\n' \
  "$VENV_DIR" "$VLLM_SRC_DIR" "$CUDA_HOME" "$MODEL_DIR" "$HF_CACHE_DIR" "$OUTPUT_DIR" \
  "$VLLM_INSTALL_MODE" "$VLLM_WHEEL_VERSION"
