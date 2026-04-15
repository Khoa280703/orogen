const elements = {
  statusLine: document.querySelector("#status-line"),
  modelSelect: document.querySelector("#model-select"),
  modeSelect: document.querySelector("#mode-select"),
  systemPrompt: document.querySelector("#system-prompt"),
  temperatureInput: document.querySelector("#temperature-input"),
  maxTokensInput: document.querySelector("#max-tokens-input"),
  thinkingToggle: document.querySelector("#thinking-toggle"),
  clearChatButton: document.querySelector("#clear-chat-button"),
  messages: document.querySelector("#messages"),
  chatForm: document.querySelector("#chat-form"),
  userInput: document.querySelector("#user-input"),
  sendButton: document.querySelector("#send-button"),
  requestState: document.querySelector("#request-state"),
  messageTemplate: document.querySelector("#message-template"),
};

const state = {
  config: null,
  models: [],
  history: [],
  pending: false,
};

const MODE_PRESETS = {
  deep: {
    thinking: true,
    temperature: 0.6,
    maxTokens: 4096,
  },
  fast: {
    thinking: false,
    temperature: 0.4,
    maxTokens: 1024,
  },
};

function createMessage(role, content, displayContent = content) {
  return { role, content, displayContent };
}

function formatAssistantDisplay(reasoning, content, isStreaming = false) {
  const safeReasoning = reasoning.trim();
  const safeContent = content.trim();

  if (safeReasoning && safeContent) {
    return `=== Thinking ===\n${safeReasoning}\n\n=== Final ===\n${safeContent}`;
  }

  if (safeContent) {
    return safeContent;
  }

  if (safeReasoning) {
    if (isStreaming) {
      return `=== Thinking ===\n${safeReasoning}\n\n[Đang stream final answer...]`;
    }
    return `=== Thinking ===\n${safeReasoning}\n\n[Chưa có final answer. Tăng Max tokens nếu anh muốn model đi hết phần trả lời.]`;
  }

  if (isStreaming) {
    return "[Đang chờ token đầu tiên...]";
  }

  return "[empty response]";
}

function setBusy(isBusy, label) {
  state.pending = isBusy;
  elements.sendButton.disabled = isBusy;
  elements.userInput.disabled = isBusy;
  elements.requestState.textContent = label;
}

function updateStatusLine() {
  const currentModel = state.models.find((model) => model.id === elements.modelSelect.value);
  const maxContext = currentModel?.max_model_len ?? "?";
  const mode = elements.modeSelect.value;
  elements.statusLine.textContent =
    `Upstream: ${state.config.upstreamBaseUrl} | Context tối đa: ${maxContext} | Mode: ${mode}`;
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

function setMode(mode) {
  elements.modeSelect.value = mode;
  applyModePreset(mode);
}

function markModeCustom() {
  if (elements.modeSelect.value !== "custom") {
    elements.modeSelect.value = "custom";
    updateStatusLine();
  }
}

function renderMessages() {
  elements.messages.innerHTML = "";
  for (const message of state.history) {
    const node = elements.messageTemplate.content.firstElementChild.cloneNode(true);
    node.dataset.role = message.role;
    node.querySelector(".message-role").textContent = message.role;
    node.querySelector(".message-content").textContent = message.displayContent;
    elements.messages.appendChild(node);
  }
  elements.messages.scrollTop = elements.messages.scrollHeight;
}

function addMessage(role, content, displayContent = content) {
  state.history.push(createMessage(role, content, displayContent));
  renderMessages();
}

function updateLastMessage(updates) {
  const lastMessage = state.history.at(-1);
  if (!lastMessage) {
    return;
  }

  Object.assign(lastMessage, updates);
  renderMessages();
}

async function loadConfig() {
  const [configResponse, modelResponse] = await Promise.all([
    fetch("/api/config"),
    fetch("/api/models"),
  ]);

  if (!configResponse.ok || !modelResponse.ok) {
    throw new Error("Không lấy được cấu hình từ server UI.");
  }

  state.config = await configResponse.json();
  const modelsPayload = await modelResponse.json();
  state.models = modelsPayload.data ?? [];

  elements.modelSelect.innerHTML = "";
  for (const model of state.models) {
    const option = document.createElement("option");
    option.value = model.id;
    option.textContent = model.id;
    if (model.id === state.config.defaultModel) {
      option.selected = true;
    }
    elements.modelSelect.appendChild(option);
  }

  if (!elements.modelSelect.value && state.models[0]) {
    elements.modelSelect.value = state.models[0].id;
  }

  updateStatusLine();
}

async function sendChat(event) {
  event.preventDefault();
  const userText = elements.userInput.value.trim();
  if (!userText || state.pending) {
    return;
  }

  addMessage("user", userText);
  elements.userInput.value = "";
  setBusy(true, "Đang chờ phản hồi...");

  const messages = [];
  const systemPrompt = elements.systemPrompt.value.trim();
  if (systemPrompt) {
    messages.push({ role: "system", content: systemPrompt });
  }

  for (const message of state.history) {
    if (message.role === "user") {
      messages.push({ role: message.role, content: message.content });
      continue;
    }

    if (message.role === "assistant" && message.content) {
      messages.push({ role: message.role, content: message.content });
    }
  }

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

  try {
    const response = await fetch("/api/chat", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
    if (!response.ok) {
      const errorText = await response.text();
      let errorMessage = "Request thất bại.";
      try {
        const errorPayload = JSON.parse(errorText);
        errorMessage = errorPayload?.error?.message || errorPayload?.error || errorMessage;
      } catch {
        if (errorText.trim()) {
          errorMessage = errorText.trim();
        }
      }
      throw new Error(errorMessage);
    }

    if (!response.body) {
      throw new Error("Browser không hỗ trợ streaming response.");
    }

    addMessage("assistant", "", formatAssistantDisplay("", "", true));
    elements.requestState.textContent = "Đang stream...";

    const reader = response.body.getReader();
    const decoder = new TextDecoder();
    let buffer = "";
    let reasoningBuffer = "";
    let contentBuffer = "";
    let usage = null;

    while (true) {
      const { value, done } = await reader.read();
      if (done) {
        break;
      }

      buffer += decoder.decode(value, { stream: true });
      const events = buffer.split("\n\n");
      buffer = events.pop() ?? "";

      for (const rawEvent of events) {
        const lines = rawEvent
          .split("\n")
          .filter((line) => line.startsWith("data:"))
          .map((line) => line.slice(5).trim());

        for (const line of lines) {
          if (!line) {
            continue;
          }

          if (line === "[DONE]") {
            continue;
          }

          let chunk;
          try {
            chunk = JSON.parse(line);
          } catch {
            continue;
          }

          const choice = chunk?.choices?.[0];
          const delta = choice?.delta ?? {};
          if (typeof delta.reasoning === "string") {
            reasoningBuffer += delta.reasoning;
          }
          if (typeof delta.content === "string") {
            contentBuffer += delta.content;
          }
          if (chunk?.usage) {
            usage = chunk.usage;
          }

          updateLastMessage({
            content: contentBuffer.trim(),
            displayContent: formatAssistantDisplay(reasoningBuffer, contentBuffer, true),
          });
        }
      }
    }

    updateLastMessage({
      content: contentBuffer.trim(),
      displayContent: formatAssistantDisplay(reasoningBuffer, contentBuffer, false),
    });
    const doneLabel = usage
      ? `Xong. prompt=${usage.prompt_tokens ?? "?"}, total=${usage.total_tokens ?? "?"}`
      : "Xong (stream).";
    setBusy(false, doneLabel);
  } catch (error) {
    addMessage("system", `Lỗi: ${error.message}`);
    setBusy(false, "Có lỗi.");
  }
}

function clearChat() {
  state.history = [];
  renderMessages();
  elements.requestState.textContent = "Đã xóa hội thoại.";
}

elements.chatForm.addEventListener("submit", sendChat);
elements.clearChatButton.addEventListener("click", clearChat);
elements.modelSelect.addEventListener("change", updateStatusLine);
elements.modeSelect.addEventListener("change", () => applyModePreset(elements.modeSelect.value));
elements.temperatureInput.addEventListener("input", markModeCustom);
elements.maxTokensInput.addEventListener("input", markModeCustom);
elements.thinkingToggle.addEventListener("change", markModeCustom);

loadConfig()
  .then(() => {
    setMode(elements.modeSelect.value || "deep");
    addMessage("system", "UI đã sẵn sàng. Deep giữ thinking mặc định; nếu chỉ thấy reasoning mà chưa có final answer thì tăng Max tokens.");
    setBusy(false, "Sẵn sàng");
  })
  .catch((error) => {
    addMessage("system", `Không khởi tạo được UI: ${error.message}`);
    setBusy(false, "Lỗi khởi tạo");
  });
