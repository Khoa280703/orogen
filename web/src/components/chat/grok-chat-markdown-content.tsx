"use client";

import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { ChatLoadingDots } from "@/components/chat/chat-loading-dots";
import type { ChatContentVariant } from "@/components/chat/chat-provider-markdown-registry";
import { cn } from "@/lib/utils";

interface GrokChatMarkdownContentProps {
  content: string;
  streaming?: boolean;
  variant?: ChatContentVariant;
}

function sanitizeGrokDisplayContent(content: string) {
  return content
    .replace(/<\/?(?:xai|grok):[^>]*>/gi, "")
    .replace(/<\/?argument[^>]*>/gi, "")
    .replace(/<!\[CDATA\[[\s\S]*?\]\]>/gi, "")
    .replace(/<(?:(?:xai|grok):|argument)[^>]*$/gi, "")
    .trim();
}

export function GrokChatMarkdownContent({
  content,
  streaming = false,
  variant = "assistant",
}: GrokChatMarkdownContentProps) {
  const sanitizedContent = sanitizeGrokDisplayContent(content);

  return (
    <div
      className={cn(
        "min-w-0 break-words text-[15px] leading-7",
        variant === "assistant" && "text-[#e2e2e2]",
        variant === "user" && "text-[#e2e2e2]",
        variant === "thinking" && "text-[#c6c6c6]",
        "[&_a]:text-white [&_a]:underline [&_a]:underline-offset-4",
        "[&_blockquote]:border-l [&_blockquote]:border-white/15 [&_blockquote]:pl-4 [&_blockquote]:italic [&_blockquote]:text-[#b6b6b6]",
        "[&_code]:rounded [&_code]:bg-white/8 [&_code]:px-1.5 [&_code]:py-0.5 [&_code]:font-mono [&_code]:text-[0.92em]",
        "[&_em]:italic [&_hr]:my-6 [&_hr]:border-white/10 [&_li]:ml-5 [&_li]:list-disc",
        "[&_ol]:ml-5 [&_ol]:list-decimal [&_p]:my-0 [&_pre]:overflow-x-auto [&_pre]:rounded-2xl [&_pre]:border [&_pre]:border-white/10 [&_pre]:bg-[#121212] [&_pre]:p-4",
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
        {sanitizedContent}
      </ReactMarkdown>

      {streaming ? (
        <ChatLoadingDots
          className="ml-2 inline-flex align-middle text-[#bfbfbf]"
          dotClassName="h-1.5 w-1.5"
        />
      ) : null}
    </div>
  );
}
