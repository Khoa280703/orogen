"use client";

import { useEffect, useRef, useState } from "react";
import { ImageIcon, Sparkles } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger } from "@/components/ui/select";
import type { ImageModelOption } from "@/lib/user-api";

interface ImagePromptBarProps {
  models: ImageModelOption[];
  selectedModel: string;
  loading?: boolean;
  onModelChange: (value: string) => void;
  onSubmit: (prompt: string) => Promise<void>;
}

export function ImagePromptBar({
  models,
  selectedModel,
  loading = false,
  onModelChange,
  onSubmit,
}: ImagePromptBarProps) {
  const [prompt, setPrompt] = useState("");
  const inputRef = useRef<HTMLTextAreaElement | null>(null);

  useEffect(() => {
    const element = inputRef.current;
    if (!element) {
      return;
    }

    element.style.height = "0px";
    element.style.height = `${Math.min(element.scrollHeight, 144)}px`;
  }, [prompt]);

  async function handleSubmit() {
    const trimmed = prompt.trim();
    if (!trimmed || loading) return;
    await onSubmit(trimmed);
    setPrompt("");
  }

  return (
    <div className="fixed bottom-0 left-0 right-0 z-40 bg-[#131313]/60 p-6 backdrop-blur-[24px] md:left-72">
      <div className="mx-auto flex w-full max-w-4xl flex-col gap-3">
        <div className="rounded-2xl border border-white/10 bg-[#0e0e0e] px-2 py-3 shadow-2xl transition-all focus-within:border-white/25">
          <div className="flex items-center justify-between gap-4 px-2">
            <div className="flex min-w-0 flex-1 items-end gap-3">
              <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-white/5 text-[#c6c6c6]">
                <ImageIcon className="h-4.5 w-4.5" />
              </div>
              <textarea
                ref={inputRef}
                value={prompt}
                onChange={(event) => setPrompt(event.target.value)}
                rows={1}
                onKeyDown={(event) => {
                  if ((event.metaKey || event.ctrlKey) && event.key === "Enter") {
                    event.preventDefault();
                    void handleSubmit();
                  }
                }}
                disabled={loading}
                placeholder="Describe the image you want, style, camera, lighting, brand mood…"
                className="min-h-[24px] max-h-36 min-w-0 flex-1 resize-none overflow-y-auto bg-transparent py-2 text-sm leading-6 text-white outline-none placeholder:text-[#c6c6c6]/55"
              />
            </div>

            <div className="flex shrink-0 items-center gap-2">
              <Select
                value={selectedModel}
                onValueChange={(value) => {
                  if (value) onModelChange(value);
                }}
              >
                <SelectTrigger className="h-10 w-auto min-w-0 max-w-[12rem] rounded-xl border border-white/10 bg-[#141414] px-3 text-xs text-[#e2e2e2] shadow-[inset_0_1px_0_rgba(255,255,255,0.03)] hover:bg-[#1a1a1a]">
                  <span className="truncate pr-0.5 font-medium text-[#f2f2f2]">
                    {models.find((model) => model.id === selectedModel)?.label || "Select model"}
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
                  {models.map((model) => (
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
                disabled={loading || !prompt.trim()}
                className="rounded-xl bg-white p-2.5 text-[#1a1c1c] transition-all hover:bg-[#c8c6c5] active:scale-90"
                aria-label="Generate image"
              >
                <Sparkles className="h-4 w-4" />
              </Button>
            </div>
          </div>
        </div>

        <p className="text-center text-[9px] font-medium uppercase tracking-widest text-white/40">
          Image Studio · Cmd/Ctrl + Enter to generate
        </p>
      </div>
    </div>
  );
}
