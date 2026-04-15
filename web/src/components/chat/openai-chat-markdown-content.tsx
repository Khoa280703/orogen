"use client";

import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { ChatLoadingDots } from "@/components/chat/chat-loading-dots";
import type { ChatContentVariant } from "@/components/chat/chat-provider-markdown-registry";
import { cn } from "@/lib/utils";

interface OpenAiChatMarkdownContentProps {
  content: string;
  streaming?: boolean;
  variant?: ChatContentVariant;
}

export function OpenAiChatMarkdownContent({
  content,
  streaming = false,
  variant = "assistant",
}: OpenAiChatMarkdownContentProps) {
  return (
    <div
      className={cn(
        "min-w-0 break-words text-[15px] leading-7",
        variant === "assistant" && "text-[#ececec]",
        variant === "user" && "text-[#ececec]",
        variant === "thinking" && "text-[#bfc7d4]",
        "[&_a]:text-sky-300 [&_a]:underline [&_a]:underline-offset-4",
        "[&_blockquote]:border-l [&_blockquote]:border-sky-400/20 [&_blockquote]:pl-4 [&_blockquote]:italic [&_blockquote]:text-[#bfc7d4]",
        "[&_code]:rounded-md [&_code]:bg-[#20252d] [&_code]:px-1.5 [&_code]:py-0.5 [&_code]:font-mono [&_code]:text-[0.92em]",
        "[&_em]:italic [&_hr]:my-6 [&_hr]:border-white/10 [&_li]:ml-5 [&_li]:list-disc",
        "[&_ol]:ml-5 [&_ol]:list-decimal [&_p]:my-0 [&_pre]:overflow-x-auto [&_pre]:rounded-2xl [&_pre]:border [&_pre]:border-white/10 [&_pre]:bg-[#14181f] [&_pre]:p-4",
        "[&_pre_code]:bg-transparent [&_pre_code]:p-0 [&_strong]:font-semibold [&_table]:w-full [&_table]:border-collapse",
        "[&_td]:border-b [&_td]:border-white/10 [&_td]:px-3 [&_td]:py-2 [&_th]:border-b [&_th]:border-white/10 [&_th]:px-3 [&_th]:py-2 [&_th]:text-left",
        "[&_ul]:ml-5 [&_ul]:list-disc"
      )}
    >
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          p: ({ children }) => <p className="mb-4 last:mb-0">{children}</p>,
          ul: ({ children }) => <ul className="mb-4 space-y-2 last:mb-0">{children}</ul>,
          ol: ({ children }) => <ol className="mb-4 space-y-2 last:mb-0">{children}</ol>,
          li: ({ children }) => <li>{children}</li>,
          pre: ({ children }) => <pre>{children}</pre>,
          code: ({ children, className }) => (
            <code className={cn(className, "font-mono")}>{children}</code>
          ),
        }}
      >
        {content}
      </ReactMarkdown>

      {streaming ? (
        <ChatLoadingDots
          className="ml-2 inline-flex align-middle text-sky-300/80"
          dotClassName="h-1.5 w-1.5"
        />
      ) : null}
    </div>
  );
}
