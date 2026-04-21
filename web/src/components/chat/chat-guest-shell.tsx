"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { ChatErrorBanner } from "@/components/chat/chat-error-banner";
import { ChatConversationHeader } from "@/components/chat/chat-conversation-header";
import { ChatInput } from "@/components/chat/chat-input";
import { ChatMessageList } from "@/components/chat/chat-message-list";
import type { ChatModelOption } from "@/lib/user-api";

interface ChatGuestShellProps {
  chatModels: ChatModelOption[];
  error?: string | null;
  onDismissError?: () => void;
  onSubmit?: (content: string, selectedChatModel: string) => Promise<void>;
}

const GUEST_MESSAGES: never[] = [];

export function ChatGuestShell({
  chatModels,
  error = null,
  onDismissError,
  onSubmit,
}: ChatGuestShellProps) {
  const router = useRouter();
  const [selectedChatModel, setSelectedChatModel] = useState(
    chatModels[0]?.id || ""
  );
  const effectiveSelectedChatModel = selectedChatModel || chatModels[0]?.id || "";

  async function handleSubmit(content: string) {
    if (onSubmit) {
      await onSubmit(content, effectiveSelectedChatModel);
      return;
    }

    router.push("/login");
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
      <ChatConversationHeader title="New conversation" />

      {error && onDismissError ? (
        <ChatErrorBanner message={error} onDismiss={onDismissError} />
      ) : null}

      <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
        <ChatMessageList messages={GUEST_MESSAGES} />
      </div>

      <ChatInput
        chatModels={chatModels}
        selectedChatModel={effectiveSelectedChatModel}
        onChatModelChange={setSelectedChatModel}
        onSubmit={handleSubmit}
      />
    </div>
  );
}
