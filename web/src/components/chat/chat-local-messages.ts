"use client";

import type {
  ConversationMessage,
  ImageGenerationRecord,
} from "@/lib/user-api";
import { resolveProviderFromModelId } from "@/lib/user-api";
import type { RenderableChatMessage } from "@/components/chat/chat-message";

const LOCAL_MESSAGES_PREFIX = "chat-local-messages:";
const PENDING_DRAFT_PREFIX = "chat-pending-draft:";

export interface PendingConversationDraft {
  content: string;
  mode: "chat" | "image";
  chatModel: string;
  imageModel: string;
  createdAt: string;
}

function getStorageKey(conversationId: number | string) {
  return `${LOCAL_MESSAGES_PREFIX}${conversationId}`;
}

function getMessageTimestamp(message: RenderableChatMessage) {
  if (!message.createdAt) {
    return 0;
  }

  const value = new Date(message.createdAt).getTime();
  return Number.isFinite(value) ? value : 0;
}

export function mapConversationMessages(
  messages: ConversationMessage[]
): RenderableChatMessage[] {
  return messages.map((message) => ({
    id: String(message.id),
    role: message.role === "assistant" ? "assistant" : "user",
    content: message.content,
    createdAt: message.created_at,
    provider: resolveProviderFromModelId(
      message.model_slug,
      message.provider_slug
    ),
    modelLabel: message.model_slug || undefined,
  }));
}

export function mergeRenderableMessages(
  baseMessages: RenderableChatMessage[],
  localMessages: RenderableChatMessage[]
): RenderableChatMessage[] {
  return [...baseMessages, ...localMessages].sort((left, right) => {
    return getMessageTimestamp(left) - getMessageTimestamp(right);
  });
}

export function readLocalMessages(
  conversationId: number | string
): RenderableChatMessage[] {
  if (typeof window === "undefined") {
    return [];
  }

  try {
    const raw = window.sessionStorage.getItem(getStorageKey(conversationId));
    if (!raw) {
      return [];
    }

    const parsed = JSON.parse(raw);
    return Array.isArray(parsed) ? (parsed as RenderableChatMessage[]) : [];
  } catch {
    return [];
  }
}

export function appendLocalMessages(
  conversationId: number | string,
  messages: RenderableChatMessage[]
) {
  if (typeof window === "undefined") {
    return;
  }

  const current = readLocalMessages(conversationId);
  window.sessionStorage.setItem(
    getStorageKey(conversationId),
    JSON.stringify([...current, ...messages])
  );
}

function getPendingDraftKey(conversationId: number | string) {
  return `${PENDING_DRAFT_PREFIX}${conversationId}`;
}

export function writePendingConversationDraft(
  conversationId: number | string,
  draft: PendingConversationDraft
) {
  if (typeof window === "undefined") {
    return;
  }

  window.sessionStorage.setItem(
    getPendingDraftKey(conversationId),
    JSON.stringify(draft)
  );
}

export function consumePendingConversationDraft(
  conversationId: number | string
): PendingConversationDraft | null {
  if (typeof window === "undefined") {
    return null;
  }

  try {
    const key = getPendingDraftKey(conversationId);
    const raw = window.sessionStorage.getItem(key);
    if (!raw) {
      return null;
    }

    window.sessionStorage.removeItem(key);
    const parsed = JSON.parse(raw);

    if (
      parsed &&
      typeof parsed.content === "string" &&
      (parsed.mode === "chat" || parsed.mode === "image")
    ) {
      return {
        content: parsed.content,
        mode: parsed.mode,
        chatModel:
          typeof parsed.chatModel === "string" ? parsed.chatModel : "grok-3",
        imageModel:
          typeof parsed.imageModel === "string"
            ? parsed.imageModel
            : "imagine-x-1",
        createdAt:
          typeof parsed.createdAt === "string"
            ? parsed.createdAt
            : new Date().toISOString(),
      };
    }

    return null;
  } catch {
    return null;
  }
}

export function peekPendingConversationDraft(
  conversationId: number | string
): PendingConversationDraft | null {
  if (typeof window === "undefined") {
    return null;
  }

  try {
    const raw = window.sessionStorage.getItem(getPendingDraftKey(conversationId));
    if (!raw) {
      return null;
    }

    const parsed = JSON.parse(raw);

    if (
      parsed &&
      typeof parsed.content === "string" &&
      (parsed.mode === "chat" || parsed.mode === "image")
    ) {
      return {
        content: parsed.content,
        mode: parsed.mode,
        chatModel:
          typeof parsed.chatModel === "string" ? parsed.chatModel : "grok-3",
        imageModel:
          typeof parsed.imageModel === "string"
            ? parsed.imageModel
            : "imagine-x-1",
        createdAt:
          typeof parsed.createdAt === "string"
            ? parsed.createdAt
            : new Date().toISOString(),
      };
    }

    return null;
  } catch {
    return null;
  }
}

export function createImageMessagePair(
  prompt: string,
  model: string,
  record: ImageGenerationRecord,
  provider = resolveProviderFromModelId(record.model_slug || model)
): RenderableChatMessage[] {
  const createdAt = record.created_at || new Date().toISOString();
  const baseId = `image-${record.id}-${Date.now()}`;
  const imageCount = record.images.length;

  return [
    {
      id: `${baseId}-user`,
      role: "user",
      content: prompt,
      createdAt,
      mode: "image",
      modelLabel: model,
      provider,
    },
    {
      id: `${baseId}-assistant`,
      role: "assistant",
      content:
        imageCount > 0
          ? `Generated ${imageCount} image${imageCount > 1 ? "s" : ""}.`
          : "Image generation completed.",
      createdAt,
      mode: "image",
      modelLabel: record.model_slug,
      images: record.images,
      provider,
    },
  ];
}
