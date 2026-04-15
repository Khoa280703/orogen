# Scout Report: Đọc lại codebase kỹ

## 1. scope đọc

| Component | File | Lines |
|-----------|------|-------|
| Gatewav | scripts/qwen35-4b-gateway.py | 527 |
| Chat UI Server | scripts/qwen35-chat-ui-server.py | 184 |
| Chat UI App | ui/qwen35-chat-ui/app.js | 329 |
| CSS | ui/qwen35-chat-ui/styles.css | 209 |
| HTML | ui/qwen35-chat-ui/index.html | 78 |
| README | README.md | 498 |

**Total**: ~1,726 lines

## 2. Core Components

### 27B services:

| Service | Port | Setup Script | Status |
|---------|------|--------------|--------|
| **Qwen3.5 27B AWQ** | 8002 | `setup-qwen35-27b-vllm.sh` | Hardware: GPU 0,1 |

| Service | Port | Template | GPU |
|---------|------|----------|-----|
| **LTT-3.5 GGUF** | 8005 | `setup-qwopus35-27b-q4km-gguf.sh` | GPU 0,1,2 |

### 4B services (LANE CHÍNH):

| Service | Port | GPU | Feature |
|---------|------|-----|---------|
| vLLM 4B single | 8004 | GPU 1 | 1 replica |
| Gateway cluster | 8100-8102 | GPU 0,1,2 | load balancing |

### Video generation:

| Service | Template | GPU | Preset |
|---------|----------|-----|--------|
| Wan2.2 I2V | ComfyUI | GPU 0,1,2 | 720×1280, 81f, 16fps |
| LTX-2.3 | LTT-Gen | GPU 0,1,2 | Text→Video, cần Gemma |

## 3. Architecture Layers

### Proxy Layer

```
┌─────────────────────────────────────────────────────────────┐
│                        UI Layer                             │
│  ui/qwen35-chat-ui/ (Vanilla JavaScript + HTML + CSS)      │
│  └── SSE streaming · chat history · mode presets           │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      Runtime/Proxy Layer                    │
│  scripts/qwen35-chat-ui-server.py *(Static + API proxy)    │
└─────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┼────────────────┴────────────────┐
              ▼               ▼                                ▼
┌─────────────────┐ ┌───────────────┐                 ┌────────────┐
│ vLLM 27B :8002 │ │ vLLM 4B :8004 │ │   llama-server    │
│  (AWQ INT4)     │ │  (Dense)      │ │  (GGUF Q4_K_M)    │
│                 │ │               │ │                   │
└────────┬────────┘ └────────│───────┘ └─────────┬────────┘
         │                   │                  │
         └────────┬───────────┴────────┬────────┘
                  │                    │
                  ▼                    ▼
      ┌──────────────────┐   ┌──────────────────┐
      │ Gateway cluster  │   │   single 4B  model │
      │ Port 8100-80102 │   │  https://8004       │
      │  (Token Budget)  │   │  (No token budget) │
      └──────────────────┘   └──────────────────┘
```

## 4. Gateway Logic Deep Dive

### 4.1 Token budget + Queue Mechanism

**Files**: `scripts/qwen35-4b-gateway.py`

| Line | Function | Purpose |
|------|----------|---------|
| 30-36 | `RAW_UPSTREAMS` | List phía upstream: `http://127.0.0.1:8100,8101,8102` |
| 34-36 | `RAW_UPSTREAM_BUDGETS` | 300K (port 8100) vs 400K (8101/8102) |
| 40-47 | `UpstreamState` | flaged + reserved_tokens + token_budget |
| 54-55 | `state` + `WAIT_QUEUE` | Global state + waiting jobs |
| 90-98 | Init budget | Default budget cho upstream |
| 255-311 | `_reserve_tokens_for_request()` | Main core logic |
| 314-322 | `_release_upstream()` | Release khi request hoàn tất |
| 407-437 | Request routing logic | scheduled vs immediate |

### 4.2 Flow diagram `_reserve_tokens_for_request()`

```
Request → Check path (/v1/chat/completions, ...)
  │
  ├─ Input token counting (_count_input_tokens)
  │   → Call upstream /v1/messages/count_tokens
  │   → Retry another upstream on error
  │
  ├─ Validate size (_validate_request_size)
  │   → Error nếu input_vượt 128K hoặc tổng_vượt 131K
  │
  ├─ Add to WAIT_QUEUE (STATE_COND)
  │   → purchase: input_tokens, required_tokens = input + TOKEN_BUDGET_RESERVE (default 0)
  │
  ├─ Wait loop:
  │   ├─ Find fitting upstream:
  │   │   → remaining_tokens >= required_tokens AND
  │   │   → inflight < max_live_requests
  │   ├─ If found:
  │   │   → Remove from queue
  │   │   → UPSTREAM_STATE[upstream].inflight += 1
  │   │   → UPSTREAM_STATE[upstream].reserved_tokens += required_tokens
  │   │   → Return upstream
  │   └─ If not found:
  │       → Wait (STATE_COND.wait, max 1800s total)
  │       → Timeout → raise TimeoutError
  │
  └─ Call request on upstream → _proxy_json / _proxy_stream
     → _release_upstream() (finally)
 
```

### 4.3 Load routing algorithm

**Function**: `_choose_proxy_upstream()` (line 143-157)

```python
sort key = (
    inflight,                          # 1. fewer current requests preferred
    -remaining_tokens,                 # 2. more budget left preferred
    random_seed,                       # 3. tie-breaker
)
```

This is a **harmonic mean** algorithm to balance load + token budget.

## 5. Chat UI Implementation

**Files**: `ui/qwen35-chat-ui/app.js` (329 lines)

### 5.1 Mode presets

| Mode | thinking | temperature | max_tokens |
|------|----------|-------------|------------|
| deep | true | 0.6 | 4096 |
| fast | false | 0.4 | 1024 |
| custom | user-defined | user-defined | user-defined |

### 5.2 SSE parsing (line 240-290)

```javascript
raw = response.body.getReader()
buffer = ""

while (true) {
  chunk = await reader.read()
  buffer += TextDecoder.decode(chunk)
  
  events = buffer.split("\n\n")  // SSE boundary
  buffer = events.pop()  // Keep incomplete buffer
  
  for (rawEvent of events) {
    // Strip "data: " prefix
    // Parse JSON from each line
    
    delta = choice?.delta ?? {}
    
    // Dual stream reasoning + content
    if (delta.reasoning) reasoningBuffer += delta.reasoning
    if (delta.content) contentBuffer += delta.content
    
    updateLastMessage({
      content: contentBuffer,
      displayContent: formatAssistantDisplay(reasoningBuffer, contentBuffer),
    })
  }
}
```

### 5.3 Display rendering

```javascript
formatAssistantDisplay(reasoning, content, isStreaming) {
  if (reasoning AND content) {
    return "=== Thinking ===\n" + reasoning + "\n\n" +
           "=== Final ===\n" + content
  } 
  if (content) {
    return content
  }
  // ... handling for reasoning-only
}
```

## 6. Noise Reduction

### 6.1 No Backend Processing

- **No post-processing logic** after upstream response
- API passes through unchanged with transparent headers:
  - `X-Qwen-Gateway-Upstream`
  - `X-Qwen-Gateway-Reserved-Tokens`
  - `X-Qwen-Gateway-Input-Tokens`

### 6.2 No CLI Tools

- No CLI abstraction layer
- Direct HTTP requests to upstream (urllib)
- No subagent spawning → all logic in pure Python/JS

### 6.3 No ML Pipeline

- No Jupyter/ML pipeline
- Pure inference-focused
- Setup scripts (bash) → Python → HTTP

## 7. Unresolved Questions

1. **Where is vLLM build source?**
   - Cache path: `$HOME/.cache/vllm-src`
   - But not in repo
   
2. **Model file locations**
   -保存在 `models/` nhưng không quá rõ ràng (benchmark/ comfyui/ qwen35/ qwopus35/)

3. **Gemma model**
   - Tự cung qua `LTX_GEMMA_ROOT`
   - Không bundle sẵn trong workspace

4. **CCS Profile setup**
   - Script: `scripts/setup-qwen35-ccs-profile.sh` (1427 lines)
   - Persist vào `~/.claude/settings.json`
   - Tránh path `/mnt/no-backup` (device некогда)

5. **CUDA 12 Patch path**
   - Default: `/usr/local/cuda-12.4`
   - Có thể override qua `QWEN35_CUDA_HOME`
   - Patch files trong `.cache/vllm-src`

6. **Docs folder (deprecated?)**
   - Không thấy `./docs/` folder trong CWD
   - README.md replace docs profile

## 8. Recommendations

If **scaling/optimization** needed:

1. **Gateway improvements**: Add metrics collection + per-request telemetry
2. **CI/CD pipeline**: Automated smoke tests on setup
3. **Volume dashboard**: Real-time upstream status display
4. **Monitoring**: Prometheus + Grafana for token budget usage
5. **Video gen benchmark**: Quantitative comparison Wan/LTX outputs

## 9. Files Changed/To Change

No files changed in this session.

---

## 🔗 externas links

- **vLLM source**: `$HOME/.cache/vllm-src`
- **Model refs**: `model-reference-portrait.png` (3868 lines)
- **Benchmarks**: `scripts/benchmark-*.py`
- **Reset scripts**: `reset-*.sh` for clean startup

---

*Report generated: 2026-04-15 09:42*
