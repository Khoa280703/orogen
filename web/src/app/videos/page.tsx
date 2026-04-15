"use client";

import { useEffect, useRef, useState } from "react";
import { Clapperboard, Sparkles } from "lucide-react";
import { ChatConversationHeader } from "@/components/chat/chat-conversation-header";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
} from "@/components/ui/select";
import {
  generateVideos,
  listChatModels,
  type ChatModelOption,
  type VideoGenerationRecord,
} from "@/lib/user-api";

export const dynamic = "force-dynamic";

const DEFAULT_MODELS: ChatModelOption[] = [
  { id: "grok-3", label: "Grok 3", provider: "grok" },
];

export default function VideosPage() {
  const [prompt, setPrompt] = useState("");
  const [models, setModels] = useState<ChatModelOption[]>(DEFAULT_MODELS);
  const [selectedModel, setSelectedModel] = useState("grok-3");
  const [duration, setDuration] = useState("6");
  const [resolution, setResolution] = useState("480p");
  const [generating, setGenerating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<VideoGenerationRecord | null>(null);
  const inputRef = useRef<HTMLTextAreaElement | null>(null);

  useEffect(() => {
    const element = inputRef.current;
    if (!element) {
      return;
    }

    element.style.height = "0px";
    element.style.height = `${Math.min(element.scrollHeight, 144)}px`;
  }, [prompt]);

  useEffect(() => {
    let alive = true;

    async function loadModels() {
      try {
        const nextModels = await listChatModels();
        if (!alive || !nextModels.length) return;
        setModels(nextModels);
        setSelectedModel(nextModels[0]?.id || "grok-3");
      } catch {
        // Keep fallback models.
      }
    }

    void loadModels();
    return () => {
      alive = false;
    };
  }, []);

  async function handleGenerate() {
    const trimmed = prompt.trim();
    if (!trimmed || generating) return;

    try {
      setGenerating(true);
      setError(null);
      const record = await generateVideos({
        prompt: trimmed,
        model: selectedModel,
        duration_seconds: Number(duration),
        resolution,
        mode: "custom",
      });
      setResult(record);
    } catch (nextError) {
      setError(nextError instanceof Error ? nextError.message : "Video generation failed");
    } finally {
      setGenerating(false);
    }
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
      <ChatConversationHeader />

      <div className="flex min-h-0 flex-1 flex-col overflow-y-auto px-6 pb-44 pt-24">
        <div className="mx-auto flex w-full max-w-4xl flex-col gap-6">
          <div>
            <p className="text-xs uppercase tracking-[0.18em] text-[#919191]">Video Studio</p>
            <h1 className="mt-1 text-3xl font-semibold text-white">Generate short videos outside chat</h1>
            <p className="mt-2 max-w-2xl text-sm leading-7 text-[#c6c6c6]">
              Same studio shell as chat, but dedicated to prompt-based video generation.
            </p>
          </div>

          {error ? (
            <div className="rounded-2xl border border-rose-400/20 bg-rose-400/10 px-4 py-3 text-sm text-rose-100">
              {error}
            </div>
          ) : null}

          {result?.data.length ? (
            <section className="space-y-4 rounded-[28px] border border-white/10 bg-[#171717] p-5">
              <div className="flex items-start justify-between gap-4">
                <div className="min-w-0">
                  <p className="text-xs uppercase tracking-[0.18em] text-[#919191]">Latest result</p>
                  <p className="mt-2 text-base text-white">{prompt || "Generated video"}</p>
                </div>
                <Badge variant="outline" className="border-white/10 text-slate-300">
                  {result.resolution} · {result.duration_seconds}s
                </Badge>
              </div>

              <div className="grid gap-4">
                {result.data.map((video) => (
                  <div key={video.id} className="overflow-hidden rounded-3xl border border-white/10 bg-black">
                    <video controls className="aspect-video w-full bg-black" src={video.url} />
                    <div className="flex items-center justify-between gap-3 px-4 py-3 text-sm">
                      <div>
                        <p className="font-medium text-white">{video.model_name || selectedModel}</p>
                        <p className="text-[#919191]">{video.resolution_name || result.resolution}</p>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </section>
          ) : (
            <div className="flex min-h-[42vh] flex-1 items-center justify-center">
              <div className="w-full max-w-3xl space-y-8 text-center">
                <div className="mx-auto flex h-14 w-14 items-center justify-center rounded-2xl bg-[#1f1f1f] text-white">
                  <Clapperboard className="h-6 w-6" />
                </div>
                <div>
                  <h2 className="font-[var(--font-chat-headline)] text-5xl font-extrabold tracking-tighter text-white md:text-6xl">
                    What scene should move?
                  </h2>
                  <p className="mx-auto mt-4 max-w-2xl text-sm leading-7 text-[#c6c6c6]">
                    Describe motion, camera path, lighting, and pacing. Keep chat separate from media creation.
                  </p>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>

      <div className="fixed bottom-0 left-0 right-0 z-40 bg-[#131313]/60 p-6 backdrop-blur-[24px] md:left-72">
        <div className="mx-auto flex w-full max-w-4xl flex-col gap-3">
          <div className="rounded-2xl border border-white/10 bg-[#0e0e0e] px-2 py-3 shadow-2xl transition-all focus-within:border-white/25">
            <div className="flex flex-col gap-3 px-2">
              <div className="flex min-w-0 items-end gap-3">
                <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-white/5 text-[#c6c6c6]">
                  <Clapperboard className="h-4.5 w-4.5" />
                </div>
                <textarea
                  ref={inputRef}
                  value={prompt}
                  onChange={(event) => setPrompt(event.target.value)}
                  rows={1}
                  onKeyDown={(event) => {
                    if ((event.metaKey || event.ctrlKey) && event.key === "Enter") {
                      event.preventDefault();
                      void handleGenerate();
                    }
                  }}
                  disabled={generating}
                  placeholder="Describe the shot, movement, camera language, style, lighting, and pacing…"
                  className="min-h-[24px] max-h-36 min-w-0 flex-1 resize-none overflow-y-auto bg-transparent py-2 text-sm leading-6 text-white outline-none placeholder:text-[#c6c6c6]/55"
                />
              </div>

              <div className="flex flex-wrap items-center justify-between gap-2">
                <div className="flex flex-wrap items-center gap-2">
                  <Select
                    value={selectedModel}
                    onValueChange={(value) => {
                      if (value) setSelectedModel(value);
                    }}
                  >
                    <SelectTrigger className="h-10 w-auto min-w-0 max-w-[12rem] rounded-xl border border-white/10 bg-[#141414] px-3 text-xs text-[#e2e2e2] hover:bg-[#1a1a1a]">
                      <span className="truncate pr-0.5 font-medium text-[#f2f2f2]">
                        {models.find((model) => model.id === selectedModel)?.label || "Select model"}
                      </span>
                    </SelectTrigger>
                    <SelectContent className="w-[min(17rem,calc(100vw-2rem))] overflow-hidden rounded-2xl border border-white/10 bg-[#101010] p-0 text-[#e2e2e2] shadow-[0_18px_48px_rgba(0,0,0,0.45)]">
                      {models.map((model) => (
                        <SelectItem key={model.id} value={model.id}>
                          {model.label}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>

                  <Select
                    value={duration}
                    onValueChange={(value) => {
                      if (value) setDuration(value);
                    }}
                  >
                    <SelectTrigger className="h-10 rounded-xl border border-white/10 bg-[#141414] px-3 text-xs text-[#f2f2f2] hover:bg-[#1a1a1a]">
                      <span>{duration}s</span>
                    </SelectTrigger>
                    <SelectContent className="rounded-2xl border border-white/10 bg-[#101010] text-[#e2e2e2]">
                      <SelectItem value="6">6 seconds</SelectItem>
                      <SelectItem value="10">10 seconds</SelectItem>
                    </SelectContent>
                  </Select>

                  <Select
                    value={resolution}
                    onValueChange={(value) => {
                      if (value) setResolution(value);
                    }}
                  >
                    <SelectTrigger className="h-10 rounded-xl border border-white/10 bg-[#141414] px-3 text-xs text-[#f2f2f2] hover:bg-[#1a1a1a]">
                      <span>{resolution}</span>
                    </SelectTrigger>
                    <SelectContent className="rounded-2xl border border-white/10 bg-[#101010] text-[#e2e2e2]">
                      <SelectItem value="480p">480p</SelectItem>
                      <SelectItem value="720p">720p</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <Button
                  onClick={() => void handleGenerate()}
                  disabled={generating || !prompt.trim()}
                  className="rounded-xl bg-white p-2.5 text-[#1a1c1c] transition-all hover:bg-[#c8c6c5] active:scale-90"
                  aria-label="Generate video"
                >
                  {generating ? <Clapperboard className="h-4 w-4 animate-pulse" /> : <Sparkles className="h-4 w-4" />}
                </Button>
              </div>
            </div>
          </div>

          <p className="text-center text-[9px] font-medium uppercase tracking-widest text-white/40">
            Video Studio · Cmd/Ctrl + Enter to generate
          </p>
        </div>
      </div>
    </div>
  );
}
