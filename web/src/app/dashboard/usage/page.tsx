"use client";

export const dynamic = "force-dynamic";

import { useEffect, useState } from "react";
import { Calendar } from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import {
  listConversations,
  listImageHistory,
  userApiRequest,
  type ConversationListItem,
  type ImageGenerationRecord,
} from "@/lib/user-api";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";

interface UsageStats {
  total_requests: number;
  today_requests: number;
  daily_stats: Array<{ date: string; requests: number; success: number; failed: number }>;
}

const PAGE_SIZE = 100;
const MAX_ACTIVITY_SCAN = 1000;

function isInsideWindow(value: string | null, days: number) {
  if (!value) return false;
  return Date.now() - new Date(value).getTime() <= days * 24 * 60 * 60 * 1000;
}

function buildBreakdown(
  usage: UsageStats | null,
  conversations: ConversationListItem[],
  images: ImageGenerationRecord[]
) {
  const rows = new Map<string, { requests: number; success: number; failed: number; chats: number; images: number }>();

  usage?.daily_stats.forEach((day) => {
    rows.set(day.date, { requests: day.requests, success: day.success, failed: day.failed, chats: 0, images: 0 });
  });

  conversations.forEach((item) => {
    const key = item.created_at?.slice(0, 10);
    if (!key || !rows.has(key)) return;
    rows.get(key)!.chats += 1;
  });

  images.forEach((item) => {
    const key = item.created_at?.slice(0, 10);
    if (!key || !rows.has(key)) return;
    rows.get(key)!.images += 1;
  });

  return [...rows.entries()]
    .map(([date, value]) => ({ date, ...value }))
    .sort((left, right) => left.date.localeCompare(right.date));
}

async function loadRecentConversations(days: number) {
  let offset = 0;
  const items: ConversationListItem[] = [];

  while (offset < MAX_ACTIVITY_SCAN) {
    const batch = await listConversations(PAGE_SIZE, offset);
    items.push(...batch.filter((item) => isInsideWindow(item.created_at, days)));
    const oldestItem = batch[batch.length - 1];
    if (batch.length < PAGE_SIZE || (oldestItem?.created_at && !isInsideWindow(oldestItem.created_at, days))) {
      return items;
    }
    offset += batch.length;
  }

  return items;
}

async function loadRecentImageHistory(days: number) {
  let offset = 0;
  const items: ImageGenerationRecord[] = [];

  while (offset < MAX_ACTIVITY_SCAN) {
    const batch = await listImageHistory(PAGE_SIZE, offset);
    items.push(...batch.filter((item) => isInsideWindow(item.created_at, days)));
    const oldestItem = batch[batch.length - 1];
    if (batch.length < PAGE_SIZE || (oldestItem?.created_at && !isInsideWindow(oldestItem.created_at, days))) {
      return items;
    }
    offset += batch.length;
  }

  return items;
}

export default function UsagePage() {
  const [days, setDays] = useState("7");
  const [usage, setUsage] = useState<UsageStats | null>(null);
  const [conversations, setConversations] = useState<ConversationListItem[]>([]);
  const [images, setImages] = useState<ImageGenerationRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let alive = true;

    async function load() {
      try {
        setLoading(true);
        setError(null);
        const range = Number(days);
        const [usageData, chatData, imageData] = await Promise.all([
          userApiRequest<UsageStats>(`/user/usage?days=${days}`),
          loadRecentConversations(range),
          loadRecentImageHistory(range),
        ]);
        if (!alive) return;
        setUsage(usageData);
        setConversations(chatData);
        setImages(imageData);
      } catch (nextError) {
        if (alive) {
          setError(nextError instanceof Error ? nextError.message : "Failed to load usage.");
        }
      } finally {
        if (alive) setLoading(false);
      }
    }

    void load();
    return () => {
      alive = false;
    };
  }, [days]);

  const dailyRows = buildBreakdown(usage, conversations, images);
  const dailyMax = dailyRows.length ? Math.max(...dailyRows.map((item) => item.requests), 1) : 1;
  const successRate = usage?.daily_stats.length
    ? Math.round((usage.daily_stats.reduce((sum, item) => sum + item.success, 0) / Math.max(usage.daily_stats.reduce((sum, item) => sum + item.requests, 0), 1)) * 100)
    : 0;

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
        <div>
          <h1 className="text-3xl font-bold">Studio usage</h1>
          <p className="mt-1 text-slate-400">Theo dõi requests, số cuộc chat mới, và các lượt tạo ảnh trong cùng một view.</p>
        </div>
        <div className="flex items-center gap-2">
          <Calendar className="h-4 w-4 text-slate-400" />
          <Select value={days} onValueChange={(value) => value && setDays(value)}>
            <SelectTrigger className="w-36">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="7">Last 7 days</SelectItem>
              <SelectItem value="14">Last 14 days</SelectItem>
              <SelectItem value="30">Last 30 days</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>

      {error ? (
        <div className="rounded-[var(--radius)] border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-200">
          {error}
        </div>
      ) : null}

      {loading ? (
        <div className="flex justify-center py-8">
          <div className="h-8 w-8 animate-spin rounded-full border-b-2 border-blue-400" />
        </div>
      ) : null}

      <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        {[
          { label: "API requests", value: usage?.total_requests || 0, hint: "Total in selected range" },
          { label: "Today requests", value: usage?.today_requests || 0, hint: "Current day volume" },
          { label: "Chats started", value: conversations.length, hint: "New conversations created" },
          { label: "Image runs", value: images.length, hint: `Success rate ${successRate}%` },
        ].map((item) => (
          <Card key={item.label} className="border-white/10 bg-white/[0.04] text-white">
            <CardHeader className="pb-2">
              <CardDescription className="text-slate-400">{item.label}</CardDescription>
              <CardTitle className="text-2xl">{item.value}</CardTitle>
            </CardHeader>
            <CardContent className="pt-0 text-sm text-slate-500">{item.hint}</CardContent>
          </Card>
        ))}
      </section>

      <Card className="border-white/10 bg-white/[0.04] text-white">
        <CardHeader>
          <CardTitle>Daily activity</CardTitle>
          <CardDescription>Requests remain the billing unit, while chats and image runs show studio behavior.</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex h-56 items-end justify-between gap-2">
            {dailyRows.length ? dailyRows.map((item) => (
              <div key={item.date} className="flex flex-1 flex-col items-center gap-2">
                <div className="w-full rounded-t-2xl bg-blue-500/30" style={{ height: `${(item.requests / dailyMax) * 180}px` }} />
                <span className="text-xs text-slate-500">{item.date.slice(5)}</span>
                <span className="text-xs text-slate-400">{item.chats}/{item.images}</span>
              </div>
            )) : <p className="text-sm text-slate-400">No activity in this period.</p>}
          </div>
        </CardContent>
      </Card>

      <Card className="border-white/10 bg-white/[0.04] text-white">
        <CardHeader>
          <CardTitle>Daily breakdown</CardTitle>
          <CardDescription>Chats/Images reflects studio objects created that day.</CardDescription>
        </CardHeader>
        <CardContent className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-slate-800">
                <th className="px-3 py-3 text-left text-slate-400">Date</th>
                <th className="px-3 py-3 text-right text-slate-400">Requests</th>
                <th className="px-3 py-3 text-right text-slate-400">Chats</th>
                <th className="px-3 py-3 text-right text-slate-400">Images</th>
                <th className="px-3 py-3 text-right text-slate-400">Success</th>
              </tr>
            </thead>
            <tbody>
              {dailyRows.map((item) => (
                <tr key={item.date} className="border-b border-slate-900">
                  <td className="px-3 py-3">{item.date}</td>
                  <td className="px-3 py-3 text-right">{item.requests}</td>
                  <td className="px-3 py-3 text-right text-blue-300">{item.chats}</td>
                  <td className="px-3 py-3 text-right text-emerald-300">{item.images}</td>
                  <td className="px-3 py-3 text-right">{item.requests ? `${Math.round((item.success / item.requests) * 100)}%` : "-"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </CardContent>
      </Card>
    </div>
  );
}
