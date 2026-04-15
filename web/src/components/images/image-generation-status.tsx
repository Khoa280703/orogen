"use client";

import { AlertCircle, Loader2, RefreshCcw } from "lucide-react";
import { Button } from "@/components/ui/button";

interface ImageGenerationStatusProps {
  loading: boolean;
  error: string | null;
  onRetry?: () => void;
}

export function ImageGenerationStatus({
  loading,
  error,
  onRetry,
}: ImageGenerationStatusProps) {
  if (loading) {
    return (
      <div className="rounded-3xl border border-blue-500/20 bg-blue-500/10 p-5 text-blue-100">
        <div className="flex items-center gap-3">
          <Loader2 className="h-5 w-5 animate-spin" />
          <div>
            <p className="text-sm font-medium">Generating images…</p>
            <p className="mt-1 text-sm text-blue-100/70">
              Grok image generation can take 10 to 30 seconds depending on queue and prompt complexity.
            </p>
          </div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="rounded-3xl border border-red-500/20 bg-red-500/10 p-5 text-red-100">
        <div className="flex items-start gap-3">
          <AlertCircle className="mt-0.5 h-5 w-5 shrink-0" />
          <div className="min-w-0 flex-1">
            <p className="text-sm font-medium">Generation failed</p>
            <p className="mt-1 text-sm text-red-100/75">{error}</p>
            {onRetry ? (
              <Button
                variant="outline"
                className="mt-4 border-red-400/20 bg-transparent text-red-100 hover:bg-red-500/10"
                onClick={onRetry}
              >
                <RefreshCcw className="h-4 w-4" />
                Retry
              </Button>
            ) : null}
          </div>
        </div>
      </div>
    );
  }

  return null;
}
