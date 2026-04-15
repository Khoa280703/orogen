"use client";

import { BrainCircuit, Copy, RefreshCcw, Sparkles, ThumbsUp } from "lucide-react";
import { ChatLoadingDots } from "@/components/chat/chat-loading-dots";
import { ChatMarkdownContent } from "@/components/chat/chat-markdown-content";
import {
  resolveChatProviderMeta,
  type ChatContentProvider,
} from "@/components/chat/chat-provider-markdown-registry";
import { ImageGallery } from "@/components/images/image-gallery";
import type { GeneratedImage } from "@/lib/user-api";

export interface RenderableChatMessage {
  id: string | number;
  role: "user" | "assistant";
  content: string;
  thinking?: string;
  thinkingStatus?: "streaming" | "complete";
  createdAt?: string | null;
  streaming?: boolean;
  mode?: "chat" | "image";
  modelLabel?: string;
  images?: GeneratedImage[];
  provider?: ChatContentProvider;
}

interface ChatMessageProps {
  message: RenderableChatMessage;
}

export function ChatMessage({ message }: ChatMessageProps) {
  const isUser = message.role === "user";
  const providerMeta = resolveChatProviderMeta(message.provider);

  if (isUser) {
    return (
      <article className="flex justify-end">
        <div className="max-w-[80%] rounded-lg bg-[#1b1b1b] px-6 py-4 text-[#e2e2e2] shadow-sm">
          <ChatMarkdownContent
            content={message.content}
            provider={message.provider}
            variant="user"
          />
        </div>
      </article>
    );
  }

  const showThinking = Boolean(message.thinking);
  const thinkingLive = message.thinkingStatus === "streaming";

  return (
    <article className="group flex gap-6">
      <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-[#393939] text-xs font-bold uppercase tracking-[0.16em] text-white shadow-lg shadow-black/20">
        AI
      </div>

      <div className="flex-1 space-y-6">
        {showThinking ? (
          <section className="rounded-2xl border border-white/8 bg-[#181818] px-4 py-4">
            <div className="flex items-center gap-2 text-xs uppercase tracking-[0.16em] text-[#919191]">
              {providerMeta.icon === "brain" ? (
                <BrainCircuit className="h-3.5 w-3.5" />
              ) : (
                <Sparkles className="h-3.5 w-3.5" />
              )}
              <span>
                {thinkingLive
                  ? providerMeta.thinkingStreamingLabel
                  : providerMeta.thinkingCompleteLabel}
              </span>
              {thinkingLive ? (
                <span className="inline-flex h-2 w-2 rounded-full bg-emerald-400 animate-pulse" />
              ) : null}
            </div>
            <div className="mt-3">
              <ChatMarkdownContent
                content={message.thinking || ""}
                provider={message.provider}
                streaming={thinkingLive}
                variant="thinking"
              />
            </div>
          </section>
        ) : null}

        <div className="space-y-3 text-lg leading-loose text-[#e2e2e2]">
          {message.content ? (
            <ChatMarkdownContent
              content={message.content}
              provider={message.provider}
              streaming={Boolean(message.streaming)}
              variant="assistant"
            />
          ) : (
            <p className="inline-flex items-center gap-2 text-[#919191]">
              <span>
                {message.streaming
                  ? message.thinking
                    ? "Generating response"
                    : "Waiting for response"
                  : "No content"}
              </span>
              {message.streaming ? (
                <ChatLoadingDots className="text-[#919191]" />
              ) : null}
            </p>
          )}
        </div>

        {message.images?.length ? (
          <div className="pt-1">
            <ImageGallery
              images={message.images}
              emptyTitle="No images"
              emptyDescription="The model did not return any images for this prompt."
            />
          </div>
        ) : null}

        <div className="flex items-center gap-4 opacity-0 transition-opacity group-hover:opacity-100">
          <button
            type="button"
            className="inline-flex items-center gap-2 rounded-lg bg-[#1f1f1f] px-3 py-1.5 text-xs text-[#c6c6c6] transition-colors hover:text-white"
          >
            <Copy className="h-3.5 w-3.5" />
            Copy
          </button>
          <button
            type="button"
            className="inline-flex items-center gap-2 rounded-lg bg-[#1f1f1f] px-3 py-1.5 text-xs text-[#c6c6c6] transition-colors hover:text-white"
          >
            <RefreshCcw className="h-3.5 w-3.5" />
            Regenerate
          </button>
          <button
            type="button"
            className="inline-flex items-center gap-2 rounded-lg bg-[#1f1f1f] px-3 py-1.5 text-xs text-[#c6c6c6] transition-colors hover:text-white"
          >
            <ThumbsUp className="h-3.5 w-3.5" />
          </button>
          {(message.modelLabel || message.mode === "image") && (
            <span className="text-[11px] uppercase tracking-[0.16em] text-[#919191]">
              {message.mode === "image" ? `Image · ${message.modelLabel || ""}` : message.modelLabel}
            </span>
          )}
        </div>
      </div>
    </article>
  );
}
