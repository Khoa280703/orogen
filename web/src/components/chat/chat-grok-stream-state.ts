"use client";

import type { ParsedChatStreamEvent } from "@/components/chat/chat-stream-handler";
import type { RenderableChatMessage } from "@/components/chat/chat-message";

export function createPendingGrokAssistantMessage(
  id: string,
  createdAt: string,
  modelLabel: string
): RenderableChatMessage {
  return {
    id,
    role: "assistant",
    provider: "grok",
    content: "",
    thinking: "",
    createdAt,
    streaming: true,
    thinkingStatus: "streaming",
    modelLabel,
  };
}

export function reduceGrokStreamMessage(
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
