"use client";

import { useEffect, useRef, useState } from "react";
import { Mic, Paperclip, SendHorizontal, Square } from "lucide-react";
import { ChatLoadingDots } from "@/components/chat/chat-loading-dots";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
} from "@/components/ui/select";
import type { ChatModelOption } from "@/lib/user-api";

interface ChatInputProps {
  chatModels: ChatModelOption[];
  selectedChatModel: string;
  disabled?: boolean;
  onChatModelChange: (value: string) => void;
  onSubmit: (content: string) => Promise<void>;
}

export function ChatInput({
  chatModels,
  selectedChatModel,
  disabled = false,
  onChatModelChange,
  onSubmit,
}: ChatInputProps) {
  const [content, setContent] = useState("");
  const [sending, setSending] = useState(false);
  const inputRef = useRef<HTMLTextAreaElement | null>(null);
  const activeModel = chatModels.find((item) => item.id === selectedChatModel);

  async function handleSubmit() {
    const trimmed = content.trim();
    if (!trimmed || sending || disabled) return;
    setSending(true);
    try {
      await onSubmit(trimmed);
      setContent("");
    } finally {
      setSending(false);
    }
  }

  useEffect(() => {
    const element = inputRef.current;
    if (!element) {
      return;
    }

    element.style.height = "0px";
    element.style.height = `${Math.min(element.scrollHeight, 144)}px`;
  }, [content]);

  return (
    <div className="fixed bottom-0 left-0 right-0 z-40 bg-[#131313]/60 p-6 backdrop-blur-[24px] md:left-72">
      <div className="mx-auto flex w-full max-w-4xl flex-col gap-4">
        <div className="rounded-2xl border border-white/10 bg-[#0e0e0e] px-2 py-3 shadow-2xl transition-all focus-within:border-white/25">
          <div className="flex items-center justify-between gap-4 px-2">
            <div className="flex min-w-0 flex-1 items-end gap-2">
              <button
                type="button"
                className="p-2 text-[#c6c6c6] transition-colors hover:text-white"
                aria-label="Attach file"
              >
                <Paperclip className="h-5 w-5" />
              </button>
              <button
                type="button"
                className="p-2 text-[#c6c6c6] transition-colors hover:text-white"
                aria-label="Microphone"
              >
                <Mic className="h-5 w-5" />
              </button>
              <textarea
                ref={inputRef}
                value={content}
                onChange={(event) => setContent(event.target.value)}
                rows={1}
                onKeyDown={(event) => {
                  if (event.key === "Enter" && !event.shiftKey) {
                    event.preventDefault();
                    void handleSubmit();
                  }
                }}
                disabled={disabled || sending}
                placeholder="Describe your vision..."
                className="min-h-[24px] max-h-36 min-w-0 flex-1 resize-none overflow-y-auto bg-transparent py-2 text-sm leading-6 text-white outline-none placeholder:text-[#c6c6c6]/55"
              />
            </div>

            <div className="flex shrink-0 items-center gap-2">
              <Select
                value={selectedChatModel}
                onValueChange={(value) => {
                  if (!value) return;
                  onChatModelChange(value);
                }}
              >
                <SelectTrigger className="h-10 w-auto min-w-0 max-w-[10rem] rounded-xl border border-white/10 bg-[#141414] px-3 text-xs text-[#e2e2e2] shadow-[inset_0_1px_0_rgba(255,255,255,0.03)] hover:bg-[#1a1a1a]">
                  <span className="truncate pr-0.5 font-medium text-[#f2f2f2]">
                    {activeModel?.label || "Select model"}
                  </span>
                </SelectTrigger>
                <SelectContent
                  side="top"
                  align="start"
                  sideOffset={10}
                  alignItemWithTrigger={false}
                  collisionAvoidance={{
                    side: "none",
                    align: "none",
                    fallbackAxisSide: "none",
                  }}
                  className="w-[min(17rem,calc(100vw-2rem))] overflow-hidden rounded-2xl border border-white/10 bg-[#101010] p-0 text-[#e2e2e2] shadow-[0_18px_48px_rgba(0,0,0,0.45)]"
                >
                  {chatModels.map((model) => (
                    <SelectItem
                      key={model.id}
                      value={model.id}
                      className="rounded-none px-0 py-0 text-[#d7d7d7] transition-colors [&_span]:text-inherit data-[highlighted]:bg-[#1b1b1b] data-[highlighted]:text-white data-[highlighted]:[&_span]:!text-white data-[selected]:bg-white/[0.04] focus:bg-[#1b1b1b] focus:text-white focus:[&_span]:!text-white"
                    >
                      <span className="flex min-w-0 w-full items-start px-3 py-3 pr-10">
                        <span className="min-w-0 flex-1">
                          <span className="flex items-center gap-2">
                            <span className="min-w-0 truncate font-medium">
                              {model.label}
                            </span>
                            <span className="shrink-0 rounded-full border border-white/10 bg-white/[0.03] px-2 py-1 text-[10px] text-[#9a9a9a]">
                              {model.provider}
                            </span>
                          </span>
                          {model.description ? (
                            <span className="mt-1 block max-w-full truncate text-[11px] text-[#8d8d8d]">
                              {model.description}
                            </span>
                          ) : null}
                        </span>
                      </span>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>

              <Button
                onClick={() => void handleSubmit()}
                disabled={disabled || sending || !content.trim()}
                className="rounded-xl bg-white p-2.5 text-[#1a1c1c] transition-all hover:bg-[#c8c6c5] active:scale-90"
                aria-label={
                  sending
                    ? "Processing request"
                    : activeModel
                      ? `Send with ${activeModel.label}`
                      : "Send message"
                }
              >
                {sending ? (
                  <Square className="h-4 w-4 fill-current" />
                ) : (
                  <SendHorizontal className="h-4 w-4" />
                )}
              </Button>
            </div>
          </div>
        </div>

        <div className="flex items-center justify-center gap-2 text-center text-[9px] font-medium uppercase tracking-widest text-white/40">
          {sending ? (
            <>
              <span>Processing</span>
              <ChatLoadingDots className="text-white/45" dotClassName="h-1 w-1" />
            </>
          ) : (
            <span>Powered by CURATOR AI Intelligence Engine v4.0</span>
          )}
        </div>
      </div>
    </div>
  );
}
