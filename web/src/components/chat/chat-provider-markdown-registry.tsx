"use client";

import type { ComponentType } from "react";
import { GrokChatMarkdownContent } from "@/components/chat/grok-chat-markdown-content";
import { OpenAiChatMarkdownContent } from "@/components/chat/openai-chat-markdown-content";

export type ChatContentProvider = string;
export type ChatContentVariant = "assistant" | "user" | "thinking";

export interface ChatMarkdownRendererProps {
  content: string;
  streaming?: boolean;
  variant?: ChatContentVariant;
}

interface ChatProviderRendererConfig {
  id: string;
  aliases?: string[];
  renderer: ComponentType<ChatMarkdownRendererProps>;
  meta: {
    displayName: string;
    thinkingStreamingLabel: string;
    thinkingCompleteLabel: string;
    icon: "brain" | "sparkles";
  };
}

const DEFAULT_PROVIDER_CONFIG: ChatProviderRendererConfig = {
  id: "grok",
  aliases: ["xai"],
  renderer: GrokChatMarkdownContent,
  meta: {
    displayName: "Grok",
    thinkingStreamingLabel: "Thinking live",
    thinkingCompleteLabel: "Thinking trace",
    icon: "brain",
  },
};

const PROVIDER_RENDERERS: ChatProviderRendererConfig[] = [
  DEFAULT_PROVIDER_CONFIG,
  {
    id: "openai",
    aliases: ["chatgpt", "gpt", "gpt-4", "gpt-5"],
    renderer: OpenAiChatMarkdownContent,
    meta: {
      displayName: "OpenAI",
      thinkingStreamingLabel: "Reasoning live",
      thinkingCompleteLabel: "Reasoning trace",
      icon: "sparkles",
    },
  },
  {
    id: "gemini",
    aliases: ["google"],
    renderer: OpenAiChatMarkdownContent,
    meta: {
      displayName: "Gemini",
      thinkingStreamingLabel: "Reasoning live",
      thinkingCompleteLabel: "Reasoning trace",
      icon: "sparkles",
    },
  },
  {
    id: "claude",
    aliases: ["anthropic"],
    renderer: OpenAiChatMarkdownContent,
    meta: {
      displayName: "Claude",
      thinkingStreamingLabel: "Reasoning live",
      thinkingCompleteLabel: "Reasoning trace",
      icon: "sparkles",
    },
  },
  {
    id: "qwen",
    aliases: ["qwq", "alibaba"],
    renderer: OpenAiChatMarkdownContent,
    meta: {
      displayName: "Qwen",
      thinkingStreamingLabel: "Reasoning live",
      thinkingCompleteLabel: "Reasoning trace",
      icon: "sparkles",
    },
  },
];

function normalizeProviderId(provider?: string) {
  return provider?.trim().toLowerCase() || "grok";
}

function resolveChatProviderConfig(provider?: string) {
  const normalizedProvider = normalizeProviderId(provider);

  return (
    PROVIDER_RENDERERS.find(
      (entry) =>
        entry.id === normalizedProvider ||
        entry.aliases?.includes(normalizedProvider)
    ) || DEFAULT_PROVIDER_CONFIG
  );
}

export function resolveChatMarkdownRenderer(provider?: string) {
  return resolveChatProviderConfig(provider).renderer;
}

export function resolveChatProviderMeta(provider?: string) {
  return resolveChatProviderConfig(provider).meta;
}

export function listRegisteredChatProviders() {
  return PROVIDER_RENDERERS.map(({ id, aliases = [] }) => ({ id, aliases }));
}
