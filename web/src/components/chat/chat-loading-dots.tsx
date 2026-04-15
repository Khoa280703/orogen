"use client";

import { cn } from "@/lib/utils";

interface ChatLoadingDotsProps {
  className?: string;
  dotClassName?: string;
}

export function ChatLoadingDots({
  className,
  dotClassName,
}: ChatLoadingDotsProps) {
  return (
    <span className={cn("inline-flex items-center gap-1", className)} aria-hidden="true">
      {[0, 1, 2].map((index) => (
        <span
          key={index}
          className={cn(
            "h-1.5 w-1.5 rounded-full bg-current curator-wave-dot",
            dotClassName
          )}
          style={{ animationDelay: `${index * 0.14}s` }}
        />
      ))}
    </span>
  );
}
