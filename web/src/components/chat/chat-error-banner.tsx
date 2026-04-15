"use client";

import { AlertCircle } from "lucide-react";
import { Button } from "@/components/ui/button";

interface ChatErrorBannerProps {
  message: string;
  onDismiss: () => void;
}

export function ChatErrorBanner({
  message,
  onDismiss,
}: ChatErrorBannerProps) {
  return (
    <div className="mx-auto mt-4 flex w-full max-w-4xl items-start gap-3 rounded-2xl border border-rose-500/20 bg-rose-500/10 px-4 py-3 text-sm text-rose-200">
      <AlertCircle className="mt-0.5 h-4 w-4 shrink-0" />
      <span className="flex-1">{message}</span>
      <Button variant="ghost" size="sm" className="text-rose-200 hover:bg-white/10" onClick={onDismiss}>
        Dismiss
      </Button>
    </div>
  );
}
