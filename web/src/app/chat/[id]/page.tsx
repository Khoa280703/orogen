"use client";

import { useCallback, useEffect, useState } from "react";
import { useParams } from "next/navigation";
import { Loader2 } from "lucide-react";
import { ChatConversationHeader } from "@/components/chat/chat-conversation-header";
import { ChatErrorBanner } from "@/components/chat/chat-error-banner";
import { resolveChatStreamAdapter } from "@/components/chat/chat-provider-stream-registry";
import {
  consumePendingConversationDraft,
  mapConversationMessages,
  mergeRenderableMessages,
  peekPendingConversationDraft,
  readLocalMessages,
} from "@/components/chat/chat-local-messages";
import {
  ChatInput,
} from "@/components/chat/chat-input";
import { ChatMessageList } from "@/components/chat/chat-message-list";
import { readChatStream } from "@/components/chat/chat-stream-handler";
import { type RenderableChatMessage } from "@/components/chat/chat-message";
import {
  getConversation,
  listChatModels,
  resolveProviderFromModelId,
  sendMessageStream,
  type ChatModelOption,
  type ConversationDetail,
} from "@/lib/user-api";

const CHAT_REFRESH_EVENT = "chat:conversations-changed";
const CHAT_STREAM_IDLE_TIMEOUT_MS = 15000;
const FALLBACK_CHAT_MODELS: ChatModelOption[] = [
  { id: "grok-3", label: "Grok 3", provider: "grok" },
];

export const dynamic = "force-dynamic";

function getDraftMessageIds(conversationId: string) {
  return {
    userId: `draft-user-${conversationId}`,
    assistantId: `draft-assistant-${conversationId}`,
  };
}

export default function ChatConversationPage() {
  const params = useParams<{ id: string }>();
  const conversationId = params?.id;
  const [detail, setDetail] = useState<ConversationDetail | null>(null);
  const [messages, setMessages] = useState<RenderableChatMessage[]>([]);
  const [chatModels, setChatModels] = useState<ChatModelOption[]>([]);
  const [selectedChatModel, setSelectedChatModel] = useState("grok-3");
  const [loading, setLoading] = useState(true);
  const [streaming, setStreaming] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [initialDraftHandled, setInitialDraftHandled] = useState(false);

  const loadConversation = useCallback(async (options?: { background?: boolean; clearError?: boolean }) => {
    if (!conversationId) {
      return;
    }

    const background = options?.background ?? false;
    const clearError = options?.clearError ?? !background;
    const pendingDraft = peekPendingConversationDraft(conversationId);
    const preserveOptimisticDraft = Boolean(pendingDraft);

    try {
      if (!background && !preserveOptimisticDraft) {
        setLoading(true);
      }
      if (clearError) {
        setError(null);
      }
      const [conversationDetail, nextChatModels] = await Promise.all([
        getConversation(conversationId),
        listChatModels(),
      ]);
      const localMessages = readLocalMessages(conversationId);

      setDetail(conversationDetail);
      if (!preserveOptimisticDraft) {
        setMessages(
          mergeRenderableMessages(
            mapConversationMessages(conversationDetail.messages),
            localMessages
          )
        );
      }
      setChatModels(nextChatModels);
      const nextSelectedChatModel =
        nextChatModels.find(
          (item) => item.id === conversationDetail.conversation.model_slug
        )?.id ||
        nextChatModels[0]?.id ||
        "grok-3";
      setSelectedChatModel(nextSelectedChatModel);
    } catch (nextError) {
      const message =
        nextError instanceof Error ? nextError.message : "Failed to load conversation";
      setError((current) => (background && current ? current : message));
    } finally {
      if (!background) {
        setLoading(false);
      }
    }
  }, [conversationId]);

  useEffect(() => {
    void loadConversation();
  }, [loadConversation]);

  useEffect(() => {
    setInitialDraftHandled(false);
    if (!conversationId) {
      return;
    }

    const draft = peekPendingConversationDraft(conversationId);
    if (!draft || draft.mode !== "chat") {
      return;
    }

    const provider = resolveProviderFromModelId(draft.chatModel, "grok");
    const streamAdapter = resolveChatStreamAdapter(provider);
    const { userId, assistantId } = getDraftMessageIds(conversationId);

    setSelectedChatModel(draft.chatModel);
    setMessages([
      {
        id: userId,
        role: "user",
        content: draft.content,
        createdAt: draft.createdAt,
        provider,
        modelLabel: draft.chatModel,
      },
      streamAdapter.createPendingAssistantMessage({
        id: assistantId,
        createdAt: draft.createdAt,
        modelLabel: draft.chatModel,
        provider,
      }),
    ]);
    setLoading(false);
  }, [conversationId]);

  useEffect(() => {
    if (!conversationId || loading || streaming || initialDraftHandled) {
      return;
    }

    const draft = consumePendingConversationDraft(conversationId);
    setInitialDraftHandled(true);

    if (!draft) {
      return;
    }

    setSelectedChatModel(draft.chatModel);
    void handleChatSubmit(draft.content, draft.chatModel, getDraftMessageIds(conversationId));
  }, [conversationId, initialDraftHandled, loading, streaming]);

  async function handleChatSubmit(
    content: string,
    modelOverride?: string,
    optimisticMessageIds?: { userId: string; assistantId: string }
  ) {
    if (!conversationId || streaming) {
      return;
    }

    const assistantMessageId =
      optimisticMessageIds?.assistantId || `assistant-${Date.now()}`;
    const userMessageId = optimisticMessageIds?.userId || `user-${Date.now()}`;
    const startedAt = new Date().toISOString();
    const targetModel = modelOverride || selectedChatModel;
    const targetModelOption =
      chatModels.find((item) => item.id === targetModel) ||
      FALLBACK_CHAT_MODELS.find((item) => item.id === targetModel);
    const provider = resolveProviderFromModelId(
      targetModel,
      targetModelOption?.provider
    );
    const streamAdapter = resolveChatStreamAdapter(provider);
    const abortController = new AbortController();
    let streamTimeout: ReturnType<typeof setTimeout> | null = null;

    const resetStreamTimeout = () => {
      if (streamTimeout) {
        clearTimeout(streamTimeout);
      }
      streamTimeout = setTimeout(() => {
        abortController.abort();
      }, CHAT_STREAM_IDLE_TIMEOUT_MS);
    };

    setError(null);
    setStreaming(true);
    setMessages((current) => {
      if (optimisticMessageIds) {
        return current.map((message) => {
          if (message.id === userMessageId) {
            return {
              ...message,
              content,
              createdAt: startedAt,
              provider,
              modelLabel: targetModelOption?.label || targetModel,
            };
          }

          if (message.id === assistantMessageId) {
            return streamAdapter.createPendingAssistantMessage({
              id: assistantMessageId,
              createdAt: startedAt,
              modelLabel: targetModelOption?.label || targetModel,
              provider,
            });
          }

          return message;
        });
      }

      return [
        ...current,
        {
          id: userMessageId,
          role: "user",
          content,
          createdAt: startedAt,
          provider,
          modelLabel: targetModelOption?.label || targetModel,
        },
        streamAdapter.createPendingAssistantMessage({
          id: assistantMessageId,
          createdAt: startedAt,
          modelLabel: targetModelOption?.label || targetModel,
          provider,
        }),
      ];
    });

    try {
      resetStreamTimeout();
      const response = await sendMessageStream(conversationId, content, targetModel, {
        signal: abortController.signal,
      });
      resetStreamTimeout();

      for await (const event of readChatStream(response)) {
        resetStreamTimeout();

        if (event.type === "error") {
          throw new Error(event.message);
        }

        setMessages((current) =>
          current.map((message) =>
            message.id === assistantMessageId
              ? streamAdapter.reduceMessage(message, event)
              : message
          )
        );
      }

      window.dispatchEvent(new Event(CHAT_REFRESH_EVENT));
      await loadConversation({ background: true, clearError: false });
    } catch (nextError) {
      const errorMessage =
        nextError instanceof Error
          ? nextError.name === "AbortError"
            ? "No response received from upstream in time. Check the proxy or account and try again."
            : nextError.message
          : "Failed to send message";

      setMessages((current) =>
        current.flatMap((message) => {
          if (message.id !== assistantMessageId) {
            return [message];
          }

          const hasVisibleContent =
            Boolean(message.content.trim()) || Boolean(message.thinking?.trim());

          if (!hasVisibleContent) {
            return [
              {
                ...message,
                content: `Error: ${errorMessage}`,
                streaming: false,
                thinking: "",
                thinkingStatus: undefined,
              },
            ];
          }

          return [
            {
              ...message,
              streaming: false,
              thinkingStatus: message.thinking ? "complete" : undefined,
            },
          ];
        })
      );
      setError(errorMessage);
    } finally {
      if (streamTimeout) {
        clearTimeout(streamTimeout);
      }
      setStreaming(false);
    }
  }

  async function handleSubmit(content: string) {
    await handleChatSubmit(content);
  }

  if (loading) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <Loader2 className="h-8 w-8 animate-spin text-slate-400" />
      </div>
    );
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
      <ChatConversationHeader title={detail?.conversation.title || "Untitled conversation"} />

      {error ? <ChatErrorBanner message={error} onDismiss={() => setError(null)} /> : null}

      <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
        <ChatMessageList messages={messages} loading={loading} streaming={streaming} />
      </div>

      <ChatInput
        chatModels={chatModels.length ? chatModels : FALLBACK_CHAT_MODELS}
        selectedChatModel={selectedChatModel}
        disabled={streaming}
        onChatModelChange={setSelectedChatModel}
        onSubmit={handleSubmit}
      />
    </div>
  );
}
