"use client";

import {
  resolveChatMarkdownRenderer,
  type ChatContentProvider,
  type ChatContentVariant,
} from "@/components/chat/chat-provider-markdown-registry";

interface ChatMarkdownContentProps {
  content: string;
  provider?: ChatContentProvider;
  streaming?: boolean;
  variant?: ChatContentVariant;
}

export function ChatMarkdownContent({
  provider = "grok",
  ...props
}: ChatMarkdownContentProps) {
  const Renderer = resolveChatMarkdownRenderer(provider);
  return <Renderer {...props} />;
}
