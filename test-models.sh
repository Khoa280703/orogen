#!/bin/bash

API_KEY="${API_KEY:-${GROK_API_KEY:-}}"
BASE_URL="${BASE_URL:-http://localhost:3069/v1/chat/completions}"

if [ -z "$API_KEY" ]; then
  echo "Missing API_KEY or GROK_API_KEY in environment." >&2
  exit 1
fi

# Text models to test
MODELS=(
  "grok-4-1-fast-reasoning"
  "grok-4-1-fast-non-reasoning"
  "grok-4-1-fast-multi-agent"
  "grok-4.20-0309-reasoning"
  "grok-4.20-0309-non-reasoning"
  "grok-4.20-multi-agent-0309"
  "grok-4-fast-reasoning"
  "grok-4-fast-non-reasoning"
  "grok-4"
  "grok-4-heavy"
  "grok-3"
  "grok-3-mini"
  "grok-2"
  "grok-latest"
  "grok-3-thinking"
)

echo "Testing Grok Models..."
echo "======================"
echo ""

for model in "${MODELS[@]}"; do
  response=$(curl -s -X POST "$BASE_URL" \
    -H "Authorization: Bearer $API_KEY" \
    -H "Content-Type: application/json" \
    -d "{\"model\":\"$model\",\"messages\":[{\"role\":\"user\",\"content\":\"Hello\"}]}" \
    -m 30 2>&1)

  # Check if response contains error
  if echo "$response" | grep -q '"error"'; then
    echo "❌ $model - ERROR"
    echo "   $(echo "$response" | grep -o '"message":"[^"]*"' | cut -d'"' -f4)"
  elif echo "$response" | grep -q '"choices"'; then
    echo "✅ $model - OK"
  else
    echo "⚠️  $model - UNKNOWN"
  fi
done

echo ""
echo "======================"
echo "Test complete!"
