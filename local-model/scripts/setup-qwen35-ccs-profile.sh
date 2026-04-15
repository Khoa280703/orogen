#!/usr/bin/env bash
set -euo pipefail

PROFILE_NAME="${QWEN35_CCS_PROFILE_NAME:-qwen-local}"
BASE_URL="${QWEN35_CCS_BASE_URL:-http://127.0.0.1:8004}"
BASE_MODEL_NAME="${QWEN35_CCS_BASE_MODEL:-qwen3.5-4b}"
MODEL_NAME="${QWEN35_CCS_MODEL:-${BASE_MODEL_NAME}-thinking}"
OPUS_MODEL_NAME="${QWEN35_CCS_OPUS_MODEL:-$MODEL_NAME}"
SONNET_MODEL_NAME="${QWEN35_CCS_SONNET_MODEL:-$MODEL_NAME}"
HAIKU_MODEL_NAME="${QWEN35_CCS_HAIKU_MODEL:-${BASE_MODEL_NAME}-no-thinking}"
API_KEY="${QWEN35_CCS_API_KEY:-qwen-local}"
AUTO_PERSIST="${QWEN35_CCS_PERSIST:-1}"
PROFILE_FILE="${HOME}/.ccs/${PROFILE_NAME}.settings.json"

require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Thiếu command: $cmd" >&2
    exit 1
  fi
}

require_cmd curl
require_cmd ccs

echo "==> Verifying vLLM Anthropic endpoints at $BASE_URL"

curl -fsS "${BASE_URL}/v1/models" >/dev/null
curl -fsS \
  -X POST "${BASE_URL}/v1/messages/count_tokens" \
  -H "content-type: application/json" \
  -H "anthropic-version: 2023-06-01" \
  -d "{\"model\":\"${MODEL_NAME}\",\"messages\":[{\"role\":\"user\",\"content\":\"ping\"}]}" >/dev/null

echo "==> Creating/updating CCS profile: ${PROFILE_NAME}"
ccs api create "${PROFILE_NAME}" \
  --base-url "${BASE_URL}" \
  --api-key "${API_KEY}" \
  --model "${MODEL_NAME}" \
  --yes \
  --force

python3 - <<PY
import json
from pathlib import Path

profile_path = Path(${PROFILE_FILE@Q})
data = json.loads(profile_path.read_text())
env = data.setdefault("env", {})
env["ANTHROPIC_MODEL"] = ${MODEL_NAME@Q}
env["ANTHROPIC_DEFAULT_OPUS_MODEL"] = ${OPUS_MODEL_NAME@Q}
env["ANTHROPIC_DEFAULT_SONNET_MODEL"] = ${SONNET_MODEL_NAME@Q}
env["ANTHROPIC_DEFAULT_HAIKU_MODEL"] = ${HAIKU_MODEL_NAME@Q}
profile_path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\\n")
PY

if [[ "${AUTO_PERSIST}" == "1" || "${AUTO_PERSIST}" == "true" ]]; then
  echo "==> Persisting ${PROFILE_NAME} into ~/.claude/settings.json"
  ccs persist "${PROFILE_NAME}" --yes
fi

echo
echo "==> Done"
echo "Profile : ${PROFILE_NAME}"
echo "Base URL: ${BASE_URL}"
echo "Model   : ${MODEL_NAME}"
echo "Opus    : ${OPUS_MODEL_NAME}"
echo "Sonnet  : ${SONNET_MODEL_NAME}"
echo "Haiku   : ${HAIKU_MODEL_NAME}"
echo
echo "Quick test:"
echo "  claude -p --model ${MODEL_NAME} --dangerously-skip-permissions 'In đúng 1 dòng: ok-local-qwen'"
