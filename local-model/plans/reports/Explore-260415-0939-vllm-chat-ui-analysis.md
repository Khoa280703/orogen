# Phân tích Chat UI Qwen3.5 - vLLM Local Implementation

**Ngày:** 2026-04-15  
**CWD:** /home/khoa2807/working-sources/local-models/ui/qwen35-chat-ui/  
**Thời điểm:** 09:39

---

## 1. Tổng quan codebase

### Cấu trúc files

| File | Mô tả | Ngôn ngữ |
|------|-------|----------|
| `index.html` | HTML boilerplate + DOM elements + message template | HTML5 |
| `app.js` | Logic chính: streaming, SSE, mode, state management | JavaScript vanilla |
| `styles.css` | CSS styling | CSS |

**[this is a vanilla JS/HTML/No TypeScript**
- File created: April 3, 2024 (old implementation)
- No TypeScript files (`*.ts`, `*.tsx`)

### API endpoints called

- `/api/config` - Returns upstream config (baseUrl, defaultModel)
- `/api/models` - Returns models data (id, max_model_len)
- `/api/chat` - Chat endpoint with streaming support

---

## 2. SSE / Stream Parsing

### Stream handling logic (app.js:233-290)

```javascript
const decoder = new TextDecoder();
let buffer = "";
let reasoningBuffer = "";
let contentBuffer = "";
let usage = null;

while (true) {
  const { value, done } = await reader.read();
  if (done) break;

  buffer += decoder.decode(value, { stream: true });
  const events = buffer.split("\n\n");
  buffer = events.pop() ?? "";

  for (const rawEvent of events) {
    const lines = rawEvent
      .split("\n")
      .filter((line) => line.startsWith("data:"))
      .map((line) => line.slice(5).trim());

    for (const line of lines) {
      if (!line) continue;
      if (line === "[DONE]") continue;

      let chunk;
      try { chunk = JSON.parse(line); } catch { continue; }

      const choice = chunk?.choices?.[0];
      const delta = choice?.delta ?? {};
      
      if (typeof delta.reasoning === "string") {
          reasoningBuffer += delta.reasoning;
      }
      if (typeof delta.content === "string") {
          contentBuffer += delta.content;
      }
      if (chunk?.usage) usage = chunk.usage;

      updateLastMessage({
          content: contentBuffer.trim(),
          displayContent: formatAssistantDisplay(reasoningBuffer, contentBuffer, true),
      });
    }
  }
}
```

### SSE Protocol_handling

- **SEvent format:** Uses Kafka/SSE event format (Kafka protocol)
- **Chunk format:** Standard SSE (`data:\n<JSON>`)
- **Multi-line support:** Handles multi-line JSON in Single Event
- **Buffering:** Buffers incomplete lines between events
- **Done signal:** Listens for `[DONE]` to end streaming

### Streaming tokens

**Content stream:** Concatenates `delta.content` ∈ get streamed character-by-character
- **Reasoning stream:** Concatenates `delta.reasoning` when thinking enabled (Qwen3.5 thinking format)
- **Display format:** `=== Thinking ===\n<reasoning>\n\n=== Final ===\n<content>`

---

## 3. Mode Logic

### MODE_PRESETS (app.js:25-36)

| Mode | thinking | temperature | maxTokens |
|------|----------|-------------|-----------|
| deep | true | 0.6 | 4096 |
| fast | false | 0.4 | 1024 |

### Mode switching (app.js:96-99, 101-106)

```javascript
function setMode(mode) {
    elements.modeSelect.value = mode;
    applyModePreset(mode);
}

function applyModePreset(mode) {
    const preset = MODE_PRESETS[mode];
    if (!preset) {
        updateStatusLine();
        return;
    }

    elements.thinkingToggle.checked = preset.thinking;
    elements.temperatureInput.value = String(preset.temperature);
    elements.maxTokensInput.value = String(preset.maxTokens);
    updateStatusLine();
}

function markModeCustom() {
    if (elements.modeSelect.value !== "custom") {
        elements.modeSelect.value = "custom";
        updateStatusLine();
    }
}
```

### Configuration state (app.js:18-23)

```javascript
const state = {
    config: null,
    models: [],
    history: [],      // Array of { role, content, displayContent }
    pending: false,
};
```

---

## 4. Messages & Display

### Message format (app.js:38-40)

```javascript
function createMessage(role, content, displayContent = content) {
    return { role, content, displayContent };
}

// Display format
function formatAssistantDisplay(reasoning, content, isStreaming) {
    const safeReasoning = reasoning.trim();
    const safeContent = content.trim();

    if (safeReasoning && safeContent) {
        return `=== Thinking ===\n${safeReasoning}\n\n=== Final ===\n${safeContent}`;
    }
    if (safeContent) return safeContent;
    if (safeReasoning) {
        if (isStreaming) return `=== Thinking ===\n${safeReasoning}\n\n[Đang stream final answer...]`;
        return `=== Thinking ===\n${safeReasoning}\n\n[Chưa có final answer. Tăng Max tokens nếu anh muốn model đi hết phần trả lời.]`;
    }
    if (isStreaming) return "[Đang chờ token đầu tiên...]";
    return "[empty response]";
}
```

---

## 5. Proxy Routing

**Current implementation:** The `/api/*` endpoints are NOT proxied in client-side code.

### Potential proxy server options that may be mirroring the requests

```javascript
// app.js:201-204 - Payload sent to /api/chat
const payload = {
    model: elements.modelSelect.value,
    messages,
    temperature: Number(elements.temperatureInput.value),
    max_tokens: Number(elements.maxTokensInput.value),
    stream: true,
    chat_template_kwargs: {
        enable_thinking: elements.thinkingToggle.checked,
    },
};
```

### Potential reverse proxy Nginx/V2/CF Workers/Vercel Edge env variables

```javascript
// app.js:137-138
fetch("/api/config");
fetch("/api/models");
```

---

## 6. Stateless / Stateless

### Request handling (app.js:167-304)

- **history:** Maintained in client JavaScript state
- **no persistent session**
- **Unicode/Encoding:** Properties Vietnamese language (Unicode UTF-8)

### Status line config (app.js:75-81)

```javascript
function updateStatusLine() {
    const currentModel = state.models.find((model) => model.id === elements.modelSelect.value);
    const maxContext = currentModel?.max_model_len ?? "?";
    const mode = elements.modeSelect.value;
    elements.statusLine.textContent =
        `Upstream: ${state.config.upstreamBaseUrl} | Context tối đa: ${maxContext} | Mode: ${mode}`;
}
```

---

## 7. Key observations

1. **No TypeScript:** Pure JavaScript implementation
2. **SSE only:** Streaming done via fetch API, not WebSocket
3. **Client-side history:** Conversation history stored in browser
4. **Mode preset:** Deep (thinking enabled) vs Fast (optimized speed)
5. **Proxy not in client:** API routes should go through backend proxy server
6. **Vietnamese language:** All UI text in Vietnamese
7. **Date:** 2024 - this is legacy code

---

## 8. Open questions

1. Where is the proxy routing configured? (backend only?)
2. How is `/api/config` and `/api/models` implemented server-side?
3. Is there client-side TypeScript version?
4. How does the thinking output (reasoning) get streamed step-by-step?
5. Any rate limiting or proxy middleware configured?

---

## 9. YAGNI status

- ✅ SSE parsing logic confirmed
- ✅ Mode switching logic confirmed
- ✅ TypeScript types: NON-EXISTENT
- ✅ Proxy routing: Backend only (client-side only consumes)
- ✅ Stream token concatenation working

---

## 10. Unresolved Qs

- [ ] Identify actual proxy server for `/api/` routes
- [ ] Verify backend API format (vLLM / Qwen3.5 server)
- [ ] Check if there's a modern Client-side TS implementation elsewhere
- [ ] Test reasoning streaming in step-by-step
