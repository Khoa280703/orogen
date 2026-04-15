"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { ArrowLeft } from "lucide-react";
import { ImageHistoryList } from "@/components/images/image-history-list";
import { listImageHistory, type ImageGenerationRecord } from "@/lib/user-api";

export const dynamic = "force-dynamic";

const PAGE_SIZE = 10;

export default function ImagesHistoryPage() {
  const [items, setItems] = useState<ImageGenerationRecord[]>([]);
  const [offset, setOffset] = useState(0);
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [canLoadMore, setCanLoadMore] = useState(false);

  useEffect(() => {
    let alive = true;

    async function loadInitial() {
      try {
        setLoading(true);
        const history = await listImageHistory(PAGE_SIZE, 0);
        if (!alive) return;
        setItems(history);
        setOffset(history.length);
        setCanLoadMore(history.length === PAGE_SIZE);
      } catch (nextError) {
        if (alive) {
          setError(nextError instanceof Error ? nextError.message : "Failed to load image history");
        }
      } finally {
        if (alive) {
          setLoading(false);
        }
      }
    }

    void loadInitial();
    return () => {
      alive = false;
    };
  }, []);

  async function handleLoadMore() {
    try {
      setLoadingMore(true);
      const next = await listImageHistory(PAGE_SIZE, offset);
      setItems((current) => [...current, ...next]);
      setOffset((current) => current + next.length);
      setCanLoadMore(next.length === PAGE_SIZE);
    } finally {
      setLoadingMore(false);
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-4 xl:flex-row xl:items-end xl:justify-between">
        <div>
          <p className="text-xs uppercase tracking-[0.18em] text-slate-500">Image Studio</p>
          <h1 className="mt-1 text-3xl font-semibold text-white">Generation history</h1>
          <p className="mt-2 text-sm leading-7 text-slate-400">
            Review previous prompts, completed assets, and failed attempts in one place.
          </p>
        </div>
        <Link
          href="/images"
          className="inline-flex h-8 items-center gap-2 rounded-[var(--radius)] border border-white/10 bg-white/[0.03] px-3 text-sm text-slate-100 transition hover:bg-white/[0.06]"
        >
          <ArrowLeft className="h-4 w-4" />
          Back to studio
        </Link>
      </div>

      {error ? (
        <div className="rounded-3xl border border-red-500/20 bg-red-500/10 px-6 py-4 text-sm text-red-200">
          {error}
        </div>
      ) : null}

      {loading ? (
        <div className="rounded-3xl border border-white/10 bg-white/[0.03] px-6 py-10 text-center text-sm text-slate-400">
          Loading history…
        </div>
      ) : (
        <ImageHistoryList
          items={items}
          showLoadMore={canLoadMore && !loadingMore}
          onLoadMore={() => void handleLoadMore()}
        />
      )}
    </div>
  );
}
