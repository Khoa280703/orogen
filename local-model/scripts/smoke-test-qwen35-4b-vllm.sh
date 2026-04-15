#!/usr/bin/env bash
set -euo pipefail

PORT="${QWEN35_PORT:-8004}"
BASE_URL="${QWEN35_BASE_URL:-http://127.0.0.1:${PORT}}"
MODEL_NAME="${QWEN35_SERVED_MODEL_NAME:-qwen3.5-4b}"
MAX_TOKENS="${QWEN35_SMOKE_MAX_TOKENS:-1024}"
TEMPERATURE="${QWEN35_SMOKE_TEMPERATURE:-0}"
ENABLE_THINKING="${QWEN35_SMOKE_ENABLE_THINKING:-0}"
RESPONSE_FILE="$(mktemp)"

cleanup() {
  rm -f "$RESPONSE_FILE"
}
trap cleanup EXIT

echo "==> Waiting for ${BASE_URL}/health"
for _ in $(seq 1 180); do
  if curl -fsS "${BASE_URL}/health" >/dev/null 2>&1; then
    break
  fi
  sleep 2
done

curl -fsS "${BASE_URL}/health" >/dev/null
echo "==> Sending smoke request"
if [[ "$ENABLE_THINKING" == "1" || "$ENABLE_THINKING" == "true" ]]; then
  ENABLE_THINKING_JSON=true
else
  ENABLE_THINKING_JSON=false
fi

curl -fsS \
  -H 'Content-Type: application/json' \
  "${BASE_URL}/v1/chat/completions" \
  -d @- >"$RESPONSE_FILE" <<JSON
{
  "model": "${MODEL_NAME}",
  "messages": [
    {
      "role": "user",
      "content": "Viết đúng 1 dòng cuối: FINAL: 4. Không giải thích dài."
    }
  ],
  "temperature": ${TEMPERATURE},
  "max_tokens": ${MAX_TOKENS},
  "chat_template_kwargs": {
    "enable_thinking": ${ENABLE_THINKING_JSON}
  }
}
JSON

python3 - "$RESPONSE_FILE" <<'PY'
import json
import sys
from pathlib import Path

payload = json.loads(Path(sys.argv[1]).read_text())
choices = payload.get("choices") or []
if not choices:
    raise SystemExit("Smoke test fail: upstream không trả choices.")

choice = choices[0]
message = choice.get("message") or {}
content = (message.get("content") or "").strip()
reasoning = (
    message.get("reasoning")
    or message.get("reasoning_content")
    or ""
).strip()

if not content:
    print(json.dumps(payload, ensure_ascii=False, indent=2))
    raise SystemExit("Smoke test fail: model chưa trả final content.")

if "FINAL:" not in content:
    print(json.dumps(payload, ensure_ascii=False, indent=2))
    raise SystemExit("Smoke test fail: chưa thấy final answer marker.")

print(
    json.dumps(
        {
            "model": payload.get("model"),
            "finish_reason": choice.get("finish_reason"),
            "reasoning_chars": len(reasoning),
            "content": content,
            "usage": payload.get("usage"),
        },
        ensure_ascii=False,
        indent=2,
    )
)
PY
