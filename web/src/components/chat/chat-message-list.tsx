"use client";

import { Brush, Code2, Sparkles } from "lucide-react";
import { useEffect, useRef } from "react";
import {
  ChatMessage,
  type RenderableChatMessage,
} from "@/components/chat/chat-message";

interface ChatMessageListProps {
  messages: RenderableChatMessage[];
  streaming?: boolean;
  loading?: boolean;
}

const SUGGESTIONS = [
  {
    icon: Brush,
    title: "Creative Direction",
    description: "Draft a storyboard for a neo-noir short film set in Tokyo.",
  },
  {
    icon: Code2,
    title: "Technical Design",
    description: "Optimize a React component for high-frequency data streams.",
  },
  {
    icon: Sparkles,
    title: "Asset Curation",
    description: "Generate a 4K texture set for architectural visualization.",
  },
];

export function ChatMessageList({
  messages,
  streaming = false,
  loading = false,
}: ChatMessageListProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const autoStickRef = useRef(true);

  useEffect(() => {
    const element = containerRef.current;
    if (!element) {
      return;
    }

    const updateAutoStick = () => {
      const distanceToBottom =
        element.scrollHeight - element.scrollTop - element.clientHeight;
      autoStickRef.current = distanceToBottom < 120;
    };

    updateAutoStick();
    element.addEventListener("scroll", updateAutoStick, { passive: true });

    return () => {
      element.removeEventListener("scroll", updateAutoStick);
    };
  }, []);

  useEffect(() => {
    const element = containerRef.current;
    if (!element || !autoStickRef.current) {
      return;
    }

    const behavior = streaming ? "auto" : "smooth";
    const frame = window.requestAnimationFrame(() => {
      element.scrollTo({
        top: element.scrollHeight,
        behavior,
      });
    });

    return () => {
      window.cancelAnimationFrame(frame);
    };
  }, [messages, loading, streaming]);

  if (!messages.length && !loading) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center overflow-y-auto px-6 pb-32 pt-20">
        <div className="w-full max-w-3xl space-y-8 text-center">
          <h2 className="font-[var(--font-chat-headline)] text-5xl font-extrabold tracking-tighter text-white md:text-7xl">
            How can I assist you today?
          </h2>
          <div className="mt-12 grid grid-cols-1 gap-4 text-left md:grid-cols-3">
            {SUGGESTIONS.map((item) => (
              <div
                key={item.title}
                className="group cursor-pointer rounded-xl bg-[#1f1f1f] p-6 transition-all duration-300 hover:bg-[#393939]"
              >
                <item.icon className="mb-4 h-5 w-5 text-white" />
                <h3 className="mb-2 font-bold text-white">{item.title}</h3>
                <p className="text-sm leading-relaxed text-[#c6c6c6]">
                  {item.description}
                </p>
              </div>
            ))}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div ref={containerRef} className="flex-1 overflow-y-auto px-6 pb-40 pt-24">
      <div className="mx-auto flex w-full max-w-4xl flex-col gap-10">
        {messages.map((message) => (
          <ChatMessage key={message.id} message={message} />
        ))}
        {loading ? <div className="text-sm text-[#c6c6c6]">Loading conversation…</div> : null}
        <div className="h-2 w-full" />
      </div>
    </div>
  );
}
