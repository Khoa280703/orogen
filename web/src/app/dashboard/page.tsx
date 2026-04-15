"use client";

export const dynamic = "force-dynamic";

import Link from "next/link";
import { useEffect, useState } from "react";
import { ArrowRight, CreditCard, ImageIcon, MessageSquare, Sparkles } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import {
  listConversations,
  listImageHistory,
  userApiRequest,
  type ConversationListItem,
  type ImageGenerationRecord,
} from "@/lib/user-api";

interface UserProfile {
  user: {
    id: number;
    email: string;
    name?: string;
  };
  plan?: {
    name: string;
    requests_per_day?: number;
    requests_per_month?: number;
  };
  balance: {
    amount: string;
  };
}

interface UsageStats {
  total_requests: number;
  today_requests: number;
  daily_stats: Array<{ date: string; requests: number }>;
}

function formatMoney(amount?: string) {
  return `$${Number.parseFloat(amount || "0").toFixed(2)}`;
}

function formatTimestamp(value: string | null) {
  if (!value) return "Just now";
  return new Date(value).toLocaleString();
}

export default function DashboardPage() {
  const [profile, setProfile] = useState<UserProfile | null>(null);
  const [usage, setUsage] = useState<UsageStats | null>(null);
  const [conversations, setConversations] = useState<ConversationListItem[]>([]);
  const [images, setImages] = useState<ImageGenerationRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [notice, setNotice] = useState<string | null>(null);

  useEffect(() => {
    let alive = true;

    async function load() {
      const results = await Promise.allSettled([
        userApiRequest<UserProfile>("/user/me"),
        userApiRequest<UsageStats>("/user/usage?days=7"),
        listConversations(5, 0),
        listImageHistory(4, 0),
      ]);

      if (!alive) return;

      setProfile(results[0].status === "fulfilled" ? results[0].value : null);
      setUsage(results[1].status === "fulfilled" ? results[1].value : null);
      setConversations(results[2].status === "fulfilled" ? results[2].value : []);
      setImages(results[3].status === "fulfilled" ? results[3].value : []);

      const failures = results.filter((result) => result.status === "rejected").length;
      setNotice(failures ? "Một phần dữ liệu studio chưa tải được. Bạn vẫn có thể tiếp tục làm việc." : null);
      setLoading(false);
    }

    void load();
    return () => {
      alive = false;
    };
  }, []);

  const dailyMax = usage?.daily_stats.length
    ? Math.max(...usage.daily_stats.map((item) => item.requests), 1)
    : 1;

  if (loading) {
    return (
      <div className="flex h-64 items-center justify-center">
        <div className="h-8 w-8 animate-spin rounded-full border-b-2 border-blue-400" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-4 xl:flex-row xl:items-end xl:justify-between">
        <div>
          <p className="text-xs uppercase tracking-[0.18em] text-slate-500">Studio Hub</p>
          <h1 className="mt-1 text-3xl font-semibold text-white">
            Welcome back, {profile?.user.name || profile?.user.email || "creator"}
          </h1>
          <p className="mt-2 max-w-2xl text-sm leading-7 text-slate-400">
            Start a new conversation, generate visuals, and keep an eye on usage without leaving the studio.
          </p>
        </div>
        <Badge variant="outline" className="border-white/10 text-slate-300">
          {profile?.plan?.name || "Free"} plan
        </Badge>
      </div>

      {notice ? (
        <div className="rounded-[var(--radius)] border border-amber-500/20 bg-amber-500/10 px-4 py-3 text-sm text-amber-100">
          {notice}
        </div>
      ) : null}

      <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        {[
          { label: "Today requests", value: usage?.today_requests || 0, hint: "Requests used today" },
          { label: "7-day total", value: usage?.total_requests || 0, hint: "Requests in the last 7 days" },
          { label: "Studio balance", value: formatMoney(profile?.balance?.amount), hint: "Available credit" },
          {
            label: "Monthly allowance",
            value: profile?.plan?.requests_per_month === -1 ? "Unlimited" : profile?.plan?.requests_per_month || 0,
            hint: "Current plan capacity",
          },
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

      <section className="grid gap-4 xl:grid-cols-3">
        <Link href="/chat?new=1" className="group">
          <Card className="h-full border-blue-500/20 bg-blue-500/10 text-white transition hover:border-blue-400/40 hover:bg-blue-500/15">
            <CardHeader>
              <MessageSquare className="h-5 w-5 text-blue-200" />
              <CardTitle>Start a fresh chat</CardTitle>
              <CardDescription className="text-blue-100/70">Open a clean thread for ideation, planning, or copy.</CardDescription>
            </CardHeader>
          </Card>
        </Link>
        <Link href="/images" className="group">
          <Card className="h-full border-emerald-500/20 bg-emerald-500/10 text-white transition hover:border-emerald-400/40 hover:bg-emerald-500/15">
            <CardHeader>
              <ImageIcon className="h-5 w-5 text-emerald-200" />
              <CardTitle>Generate visuals</CardTitle>
              <CardDescription className="text-emerald-100/70">Create hero shots, social creatives, and concept frames.</CardDescription>
            </CardHeader>
          </Card>
        </Link>
        <Link href="/dashboard/billing" className="group">
          <Card className="h-full border-white/10 bg-white/[0.04] text-white transition hover:border-white/20 hover:bg-white/[0.06]">
            <CardHeader>
              <CreditCard className="h-5 w-5 text-slate-200" />
              <CardTitle>Manage plan and balance</CardTitle>
              <CardDescription className="text-slate-400">Review quota, billing history, and upgrade options.</CardDescription>
            </CardHeader>
          </Card>
        </Link>
      </section>

      <section className="grid gap-4 xl:grid-cols-[1.2fr_0.8fr]">
        <Card className="border-white/10 bg-white/[0.04] text-white">
          <CardHeader>
            <CardTitle>Recent conversations</CardTitle>
            <CardDescription>Pick up where you left off or branch into a new thread.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            {conversations.length ? conversations.map((item) => (
              <Link
                key={item.id}
                href={`/chat/${item.id}`}
                className="flex items-center justify-between rounded-2xl border border-white/10 bg-slate-950/50 px-4 py-3 transition hover:border-white/20 hover:bg-slate-900"
              >
                <div className="min-w-0">
                  <p className="truncate text-sm font-medium text-slate-100">{item.title || "Untitled conversation"}</p>
                  <p className="mt-1 text-xs text-slate-500">{item.message_count} messages • {item.model_slug || "Default model"}</p>
                </div>
                <span className="text-xs text-slate-500">{formatTimestamp(item.updated_at)}</span>
              </Link>
            )) : <p className="text-sm text-slate-400">No conversations yet. Start one from Chat.</p>}
          </CardContent>
        </Card>

        <Card className="border-white/10 bg-white/[0.04] text-white">
          <CardHeader>
            <CardTitle>Recent image runs</CardTitle>
            <CardDescription>Latest prompts and outputs from the image studio.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            {images.length ? images.map((item) => (
              <Link
                key={item.id}
                href="/images/history"
                className="flex items-center gap-3 rounded-2xl border border-white/10 bg-slate-950/50 p-3 transition hover:border-white/20 hover:bg-slate-900"
              >
                <div className="flex h-14 w-14 shrink-0 items-center justify-center overflow-hidden rounded-xl bg-slate-900">
                  {item.images[0]?.url ? (
                    <img src={item.images[0].url} alt={item.prompt} className="h-full w-full object-cover" />
                  ) : (
                    <Sparkles className="h-4 w-4 text-slate-500" />
                  )}
                </div>
                <div className="min-w-0 flex-1">
                  <p className="truncate text-sm font-medium text-slate-100">{item.prompt}</p>
                  <p className="mt-1 text-xs text-slate-500">{item.images.length} images • {item.model_slug}</p>
                </div>
              </Link>
            )) : <p className="text-sm text-slate-400">No generations yet. Open Images to create your first set.</p>}
            <Link href="/images/history" className="inline-flex items-center gap-2 text-sm text-blue-300 transition hover:text-blue-200">
              View full history
              <ArrowRight className="h-4 w-4" />
            </Link>
          </CardContent>
        </Card>
      </section>

      <Card className="border-white/10 bg-white/[0.04] text-white">
        <CardHeader>
          <CardTitle>Usage trend</CardTitle>
          <CardDescription>Request volume over the last 7 days.</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex h-48 items-end justify-between gap-2">
            {usage?.daily_stats.length ? usage.daily_stats.map((day) => (
              <div key={day.date} className="flex flex-1 flex-col items-center gap-2">
                <div className="w-full rounded-t-2xl bg-blue-500/30" style={{ height: `${(day.requests / dailyMax) * 160}px` }} />
                <span className="text-xs text-slate-500">{day.date.slice(5)}</span>
              </div>
            )) : <p className="text-sm text-slate-400">No usage data yet.</p>}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
