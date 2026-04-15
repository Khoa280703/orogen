"use client";

import {
  createPendingGrokAssistantMessage,
  reduceGrokStreamMessage,
} from "@/components/chat/chat-grok-stream-state";
import type { RenderableChatMessage } from "@/components/chat/chat-message";
import type { ParsedChatStreamEvent } from "@/components/chat/chat-stream-handler";

interface PendingAssistantMessageInput {
  id: string;
  createdAt: string;
  modelLabel: string;
  provider: string;
}

interface ChatProviderStreamAdapterConfig {
  id: string;
  aliases?: string[];
  createPendingAssistantMessage: (
    input: PendingAssistantMessageInput
  ) => RenderableChatMessage;
  reduceMessage: (
    message: RenderableChatMessage,
    event: ParsedChatStreamEvent
  ) => RenderableChatMessage;
}

function createPendingDefaultAssistantMessage({
  id,
  createdAt,
  modelLabel,
  provider,
}: PendingAssistantMessageInput): RenderableChatMessage {
  return {
    id,
    role: "assistant",
    provider,
    content: "",
    thinking: "",
    createdAt,
    streaming: true,
    thinkingStatus: "streaming",
    modelLabel,
  };
}

function reduceDefaultStreamMessage(
  message: RenderableChatMessage,
  event: ParsedChatStreamEvent
): RenderableChatMessage {
  if (event.type === "token") {
    return {
      ...message,
      content: `${message.content}${event.content}`,
      streaming: true,
    };
  }

  if (event.type === "thinking") {
    return {
      ...message,
      thinking: `${message.thinking || ""}${event.content}`,
      thinkingStatus: "streaming",
      streaming: true,
    };
  }

  if (event.type === "done") {
    return {
      ...message,
      streaming: false,
      thinkingStatus: message.thinking ? "complete" : undefined,
    };
  }

  return message;
}

const DEFAULT_STREAM_ADAPTER: ChatProviderStreamAdapterConfig = {
  id: "default",
  createPendingAssistantMessage: createPendingDefaultAssistantMessage,
  reduceMessage: reduceDefaultStreamMessage,
};

const STREAM_ADAPTERS: ChatProviderStreamAdapterConfig[] = [
  {
    id: "grok",
    aliases: ["xai"],
    createPendingAssistantMessage: ({ id, createdAt, modelLabel }) =>
      createPendingGrokAssistantMessage(id, createdAt, modelLabel),
    reduceMessage: reduceGrokStreamMessage,
  },
  {
    ...DEFAULT_STREAM_ADAPTER,
    id: "openai",
    aliases: ["chatgpt", "gpt", "gpt-4", "gpt-5"],
  },
  {
    ...DEFAULT_STREAM_ADAPTER,
    id: "gemini",
    aliases: ["google"],
  },
  {
    ...DEFAULT_STREAM_ADAPTER,
    id: "claude",
    aliases: ["anthropic"],
  },
  {
    ...DEFAULT_STREAM_ADAPTER,
    id: "qwen",
    aliases: ["qwq", "alibaba"],
  },
];

function normalizeProviderId(provider?: string) {
  return provider?.trim().toLowerCase() || "grok";
}

export function resolveChatStreamAdapter(provider?: string) {
  const normalizedProvider = normalizeProviderId(provider);

  return (
    STREAM_ADAPTERS.find(
      (entry) =>
        entry.id === normalizedProvider ||
        entry.aliases?.includes(normalizedProvider)
    ) || DEFAULT_STREAM_ADAPTER
  );
}
