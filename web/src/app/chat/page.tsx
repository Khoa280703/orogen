"use client";

import { useEffect, useState } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import { Loader2 } from "lucide-react";
import { ChatGuestShell } from "@/components/chat/chat-guest-shell";
import { writePendingConversationDraft } from "@/components/chat/chat-local-messages";
import {
  createConversation,
  hasActiveUserSession,
  listChatModels,
  listConversations,
  type ChatModelOption,
} from "@/lib/user-api";

export const dynamic = "force-dynamic";

const CHAT_REFRESH_EVENT = "chat:conversations-changed";
const FALLBACK_CHAT_MODELS: ChatModelOption[] = [
  { id: "grok-3", label: "Grok 3", provider: "grok" },
];

export default function ChatIndexPage() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const [authenticated, setAuthenticated] = useState<boolean | null>(null);
  const [chatModels, setChatModels] = useState<ChatModelOption[]>([]);
  const [bootstrapping, setBootstrapping] = useState(true);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    const forceNew = searchParams.get("new") === "1";

    async function bootstrap() {
      try {
        const [sessionActive, nextChatModels] = await Promise.all([
          hasActiveUserSession(),
          listChatModels(),
        ]);
        if (cancelled) {
          return;
        }

        setAuthenticated(sessionActive);
        setChatModels(nextChatModels);

        if (!sessionActive) {
          setBootstrapping(false);
          return;
        }

        if (!forceNew) {
          const items = await listConversations(1, 0, {
            redirectOnUnauthorized: false,
          });
          if (cancelled) {
            return;
          }

          if (items[0]) {
            router.replace(`/chat/${items[0].id}`);
            return;
          }
        }
      } catch (nextError) {
        if (!cancelled) {
          const message =
            nextError instanceof Error ? nextError.message : "Failed to open chat";
          if (message === "Unauthorized") {
            setAuthenticated(false);
            return;
          }
          setError(message);
        }
      } finally {
        if (!cancelled) {
          setBootstrapping(false);
        }
      }
    }

    void bootstrap();

    return () => {
      cancelled = true;
    };
  }, [router, searchParams]);

  async function handleSubmit(
    content: string,
    selectedChatModel: string
  ) {
    if (submitting) {
      return;
    }

    try {
      setSubmitting(true);
      setError(null);

      const conversation = await createConversation(selectedChatModel, undefined, {
        redirectOnUnauthorized: false,
      });

      writePendingConversationDraft(conversation.id, {
        content,
        mode: "chat",
        chatModel: selectedChatModel,
        imageModel: "imagine-x-1",
        createdAt: new Date().toISOString(),
      });

      window.dispatchEvent(new Event(CHAT_REFRESH_EVENT));
      router.replace(`/chat/${conversation.id}`);
    } catch (nextError) {
      const message =
        nextError instanceof Error ? nextError.message : "Failed to open chat";

      if (message === "Unauthorized") {
        router.push("/login");
        return;
      }

      setError(message);
      setSubmitting(false);
    }
  }

  if (authenticated === false) {
    return (
      <ChatGuestShell
        chatModels={chatModels.length ? chatModels : FALLBACK_CHAT_MODELS}
      />
    );
  }

  if (bootstrapping) {
    return (
      <div className="flex min-h-0 flex-1 items-center justify-center bg-[#131313] p-6 text-center">
        <div className="w-full max-w-xl rounded-2xl border border-[#474747]/30 bg-[#1f1f1f] p-10 shadow-[0_24px_80px_rgba(0,0,0,0.45)]">
          <Loader2 className="mx-auto h-8 w-8 animate-spin text-slate-400" />
          <p className="mt-4 text-sm text-[#c6c6c6]">
            Loading your latest conversation.
          </p>
        </div>
      </div>
    );
  }

  return (
    <ChatGuestShell
      chatModels={chatModels.length ? chatModels : FALLBACK_CHAT_MODELS}
      error={error}
      onDismissError={() => setError(null)}
      onSubmit={handleSubmit}
    />
  );
}
