"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { ArrowRight, History, ImageIcon } from "lucide-react";
import { ChatConversationHeader } from "@/components/chat/chat-conversation-header";
import { ImageGallery } from "@/components/images/image-gallery";
import { ImageGenerationStatus } from "@/components/images/image-generation-status";
import { ImagePromptBar } from "@/components/images/image-prompt-bar";
import { Badge } from "@/components/ui/badge";
import {
  generateImages,
  listImageHistory,
  listImageModels,
  type ImageGenerationRecord,
  type ImageModelOption,
} from "@/lib/user-api";

export const dynamic = "force-dynamic";

export default function ImagesPage() {
  const [models, setModels] = useState<ImageModelOption[]>([]);
  const [selectedModel, setSelectedModel] = useState("imagine-x-1");
  const [recentHistory, setRecentHistory] = useState<ImageGenerationRecord[]>([]);
  const [activeGeneration, setActiveGeneration] = useState<ImageGenerationRecord | null>(null);
  const [loading, setLoading] = useState(true);
  const [generating, setGenerating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [retryPrompt, setRetryPrompt] = useState<string | null>(null);

  useEffect(() => {
    let alive = true;

    async function load() {
      try {
        setLoading(true);
        const [history, modelOptions] = await Promise.all([
          listImageHistory(5, 0),
          listImageModels(),
        ]);
        if (!alive) return;
        setRecentHistory(history);
        setActiveGeneration(history[0] || null);
        setModels(modelOptions);
        setSelectedModel(modelOptions[0]?.id || "imagine-x-1");
      } catch (nextError) {
        if (alive) {
          setError(nextError instanceof Error ? nextError.message : "Failed to load image studio");
        }
      } finally {
        if (alive) {
          setLoading(false);
        }
      }
    }

    void load();
    return () => {
      alive = false;
    };
  }, []);

  async function handleGenerate(prompt: string) {
    try {
      setGenerating(true);
      setError(null);
      setRetryPrompt(prompt);
      const record = await generateImages(prompt, selectedModel);
      setActiveGeneration(record);
      setRecentHistory((current) => [record, ...current.filter((item) => item.id !== record.id)].slice(0, 5));
    } catch (nextError) {
      setError(nextError instanceof Error ? nextError.message : "Image generation failed");
    } finally {
      setGenerating(false);
    }
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
      <ChatConversationHeader />

      <div className="flex min-h-0 flex-1 flex-col overflow-y-auto px-6 pb-44 pt-24">
        <div className="mx-auto flex w-full max-w-4xl flex-col gap-6">
          <div className="flex items-start justify-between gap-4">
            <div>
              <p className="text-xs uppercase tracking-[0.18em] text-[#919191]">Image Studio</p>
              <h1 className="mt-1 text-3xl font-semibold text-white">Generate visuals outside chat</h1>
              <p className="mt-2 max-w-2xl text-sm leading-7 text-[#c6c6c6]">
                Same studio shell as chat, but dedicated to image generation only.
              </p>
            </div>
            <Link
              href="/images/history"
              className="inline-flex h-9 items-center gap-2 rounded-full border border-white/10 bg-white/[0.03] px-4 text-sm text-slate-100 transition hover:bg-white/[0.06]"
            >
              <History className="size-4" />
              Full history
            </Link>
          </div>

          <ImageGenerationStatus
            loading={generating}
            error={error}
            onRetry={retryPrompt ? () => void handleGenerate(retryPrompt) : undefined}
          />

          {activeGeneration ? (
            <section className="space-y-4 rounded-[28px] border border-white/10 bg-[#171717] p-5">
              <div className="flex items-start justify-between gap-4">
                <div className="min-w-0">
                  <p className="text-xs uppercase tracking-[0.18em] text-[#919191]">Latest result</p>
                  <p className="mt-2 text-base text-white">{activeGeneration.prompt}</p>
                </div>
                <Badge variant="outline" className="border-white/10 text-slate-300">
                  {activeGeneration.model_slug}
                </Badge>
              </div>

              <ImageGallery
                images={activeGeneration.images}
                emptyTitle="No images generated"
                emptyDescription="Try another prompt or switch to a different model."
              />
            </section>
          ) : (
            <div className="flex min-h-[42vh] flex-1 items-center justify-center">
              <div className="w-full max-w-3xl space-y-8 text-center">
                <div className="mx-auto flex h-14 w-14 items-center justify-center rounded-2xl bg-[#1f1f1f] text-white">
                  <ImageIcon className="h-6 w-6" />
                </div>
                <div>
                  <h2 className="font-[var(--font-chat-headline)] text-5xl font-extrabold tracking-tighter text-white md:text-6xl">
                    What do you want to create?
                  </h2>
                  <p className="mx-auto mt-4 max-w-2xl text-sm leading-7 text-[#c6c6c6]">
                    Product shots, campaign art, thumbnails, storyboards, or visual explorations.
                  </p>
                </div>
              </div>
            </div>
          )}

          {recentHistory.length ? (
            <section className="space-y-4">
              <div className="flex items-center justify-between">
                <p className="text-xs uppercase tracking-[0.18em] text-[#919191]">Recent generations</p>
              </div>
              <div className="space-y-3">
                {recentHistory.map((item) => (
                  <button
                    key={item.id}
                    type="button"
                    onClick={() => setActiveGeneration(item)}
                    className="flex w-full items-center justify-between rounded-2xl border border-white/10 bg-[#171717] px-4 py-3 text-left transition hover:border-white/20 hover:bg-[#1d1d1d]"
                  >
                    <div className="min-w-0">
                      <p className="truncate text-sm font-medium text-white">{item.prompt}</p>
                      <p className="mt-1 text-xs text-[#919191]">
                        {item.images.length} images • {item.status}
                      </p>
                    </div>
                    <ArrowRight className="h-4 w-4 text-[#919191]" />
                  </button>
                ))}
              </div>
            </section>
          ) : null}
        </div>
      </div>

      <ImagePromptBar
        models={models.length ? models : [{ id: "imagine-x-1", label: "Imagine X1", provider: "grok" }]}
        selectedModel={selectedModel}
        loading={generating}
        onModelChange={setSelectedModel}
        onSubmit={handleGenerate}
      />
    </div>
  );
}
