"use client";

import { useMemo, useState } from "react";
import Link from "next/link";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ImageGallery } from "@/components/images/image-gallery";
import type { ImageGenerationRecord } from "@/lib/user-api";

interface ImageHistoryListProps {
  items: ImageGenerationRecord[];
  showLoadMore?: boolean;
  onLoadMore?: () => Promise<void> | void;
}

function formatDate(value: string | null) {
  if (!value) return "Just now";
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(value));
}

export function ImageHistoryList({
  items,
  showLoadMore = false,
  onLoadMore,
}: ImageHistoryListProps) {
  const [filter, setFilter] = useState<"all" | "completed" | "failed">("all");
  const filtered = useMemo(() => {
    if (filter === "all") return items;
    return items.filter((item) => item.status === filter);
  }, [filter, items]);

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between gap-3">
        <div className="flex items-center gap-2">
          {(["all", "completed", "failed"] as const).map((item) => (
            <Button
              key={item}
              size="sm"
              variant={filter === item ? "secondary" : "ghost"}
              className={filter === item ? "bg-white/10 text-white" : "text-slate-400"}
              onClick={() => setFilter(item)}
            >
              {item}
            </Button>
          ))}
        </div>
        <Link href="/images/history" className="text-sm text-slate-400 hover:text-white">
          Full history
        </Link>
      </div>

      <div className="space-y-4">
        {filtered.map((item) => (
          <article
            key={item.id}
            className="rounded-[2rem] border border-white/10 bg-white/[0.04] p-4"
          >
            <div className="flex flex-col gap-3 md:flex-row md:items-start md:justify-between">
              <div className="space-y-2">
                <div className="flex items-center gap-2">
                  <Badge variant="outline" className="border-white/10 text-slate-300">
                    {item.model_slug}
                  </Badge>
                  <Badge
                    variant="outline"
                    className={
                      item.status === "completed"
                        ? "border-emerald-400/20 text-emerald-200"
                        : "border-red-400/20 text-red-200"
                    }
                  >
                    {item.status}
                  </Badge>
                </div>
                <p className="max-w-3xl text-sm leading-7 text-slate-100">{item.prompt}</p>
              </div>
              <span className="text-xs text-slate-500">{formatDate(item.created_at)}</span>
            </div>

            <div className="mt-4">
              {item.status === "failed" ? (
                <div className="rounded-2xl border border-red-500/15 bg-red-500/10 px-4 py-3 text-sm text-red-100">
                  {item.error_message || "Generation failed"}
                </div>
              ) : (
                <ImageGallery
                  images={item.images}
                  emptyTitle="No assets returned"
                  emptyDescription="This generation completed but no image URLs were stored."
                />
              )}
            </div>
          </article>
        ))}

        {!filtered.length ? (
          <div className="rounded-3xl border border-dashed border-white/10 p-8 text-center text-sm text-slate-400">
            No history for this filter yet.
          </div>
        ) : null}
      </div>

      {showLoadMore && onLoadMore ? (
        <div className="flex justify-center pt-2">
          <Button variant="outline" onClick={() => void onLoadMore()}>
            Load more
          </Button>
        </div>
      ) : null}
    </div>
  );
}
