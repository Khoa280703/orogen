'use client';

import { useEffect, useState } from 'react';
import { Activity, AlertCircle, Server, Users } from 'lucide-react';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { getDailyUsage, getStatsOverview } from '@/lib/api';

interface Stats {
  total_accounts: number;
  active_accounts: number;
  requests_today: number;
  errors_today: number;
  total_conversations: number;
  total_image_generations: number;
}

interface DailyUsage {
  day: string;
  total: number;
  success: number;
  errors: number;
}

const statCards = [
  {
    key: 'accounts',
    title: 'Accounts',
    icon: Users,
    accent: 'text-blue-600',
    panel: 'bg-blue-50',
  },
  {
    key: 'requests',
    title: 'Requests today',
    icon: Activity,
    accent: 'text-slate-900',
    panel: 'bg-slate-100',
  },
  {
    key: 'errors',
    title: 'Errors today',
    icon: AlertCircle,
    accent: 'text-green-600',
    panel: 'bg-green-50',
  },
  {
    key: 'conversations',
    title: 'Conversations',
    icon: Users,
    accent: 'text-blue-600',
    panel: 'bg-blue-50',
  },
  {
    key: 'images',
    title: 'Image runs',
    icon: Server,
    accent: 'text-green-600',
    panel: 'bg-green-50',
  },
] as const;

export default function DashboardPage() {
  const [stats, setStats] = useState<Stats | null>(null);
  const [usage, setUsage] = useState<DailyUsage[]>([]);
  const [loading, setLoading] = useState(true);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  useEffect(() => {
    async function loadData() {
      try {
        setErrorMessage(null);
        const [statsData, usageData] = await Promise.all([
          getStatsOverview(),
          getDailyUsage(7),
        ]);
        setStats(statsData);
        setUsage(usageData);
      } catch (error) {
        setErrorMessage(error instanceof Error ? error.message : 'Failed to load dashboard data.');
      } finally {
        setLoading(false);
      }
    }
    loadData();
  }, []);

  const maxUsage = Math.max(...usage.map((item) => item.total), 1);
  const errorRate = stats?.requests_today
    ? ((stats.errors_today / stats.requests_today) * 100).toFixed(1)
    : '0.0';

  if (loading) {
    return <div className="py-10 text-sm text-slate-500">Loading dashboard...</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-3 lg:flex-row lg:items-end lg:justify-between">
        <div>
          <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-slate-500">Overview</div>
          <h1 className="mt-1 text-3xl font-semibold tracking-tight text-slate-950">Dashboard</h1>
          <p className="mt-2 max-w-2xl text-sm text-slate-600">
            Theo dõi tài khoản, lưu lượng và độ ổn định hệ thống trên cùng một màn hình đơn giản.
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Badge variant="outline">{stats?.active_accounts ?? 0} active accounts</Badge>
          <Badge variant="outline">{errorRate}% error rate</Badge>
        </div>
      </div>

      {errorMessage && (
        <div className="border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
          {errorMessage}
        </div>
      )}

      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        {statCards.map((item) => {
          const Icon = item.icon;
          const value =
            item.key === 'accounts'
              ? stats?.total_accounts ?? 0
              : item.key === 'requests'
                ? stats?.requests_today ?? 0
                : item.key === 'errors'
                  ? stats?.errors_today ?? 0
                  : item.key === 'conversations'
                    ? stats?.total_conversations ?? 0
                    : stats?.total_image_generations ?? 0;
          const detail =
            item.key === 'accounts'
              ? `${stats?.active_accounts ?? 0} active`
              : item.key === 'requests'
                ? '24 hour window'
                : item.key === 'errors'
                  ? `${errorRate}% of traffic`
                  : item.key === 'conversations'
                    ? 'Consumer chat threads'
                    : 'Consumer image requests';

          return (
            <Card key={item.key}>
              <CardHeader className="flex flex-row items-start justify-between border-b pb-3">
                <div>
                  <CardTitle className="text-sm font-semibold text-slate-600">{item.title}</CardTitle>
                </div>
                <div className={`flex h-9 w-9 items-center justify-center border ${item.panel}`}>
                  <Icon className={`h-4 w-4 ${item.accent}`} />
                </div>
              </CardHeader>
              <CardContent className="pt-4">
                <div className="text-3xl font-semibold tracking-tight text-slate-950">{value}</div>
                <p className="mt-1 text-sm text-slate-500">{detail}</p>
              </CardContent>
            </Card>
          );
        })}
      </div>

      <div className="grid gap-4 xl:grid-cols-[minmax(0,1.5fr)_minmax(20rem,0.8fr)]">
        <Card>
          <CardHeader className="border-b">
            <CardTitle>Usage, last 7 days</CardTitle>
          </CardHeader>
          <CardContent className="pt-5">
            <div className="flex h-72 items-end gap-3">
              {usage.map((day) => {
                const usageHeight = Math.max((day.total / maxUsage) * 100, day.total > 0 ? 8 : 2);
                const successShare = day.total > 0 ? (day.success / day.total) * 100 : 0;
                return (
                  <div key={day.day} className="flex flex-1 flex-col items-center gap-3">
                    <div className="flex h-full w-full items-end border bg-slate-50 px-2 py-2">
                      <div className="flex h-full w-full items-end bg-blue-100">
                        <div className="w-full bg-blue-600" style={{ height: `${usageHeight}%` }}>
                          <div className="bg-green-500" style={{ height: `${successShare}%` }} />
                        </div>
                      </div>
                    </div>
                    <div className="text-center">
                      <div className="text-xs font-medium text-slate-900">{day.total}</div>
                      <div className="text-[11px] text-slate-500">{day.day.slice(5)}</div>
                    </div>
                  </div>
                );
              })}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="border-b">
            <CardTitle>Operational notes</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4 pt-5">
            <div className="border px-4 py-3">
              <div className="text-xs font-semibold uppercase tracking-[0.16em] text-slate-500">Accounts</div>
              <div className="mt-2 text-2xl font-semibold text-slate-950">{stats?.active_accounts ?? 0}</div>
              <p className="mt-1 text-sm text-slate-600">Sẵn sàng phục vụ request ngay lúc này.</p>
            </div>
            <div className="border px-4 py-3">
              <div className="text-xs font-semibold uppercase tracking-[0.16em] text-slate-500">Errors</div>
              <div className="mt-2 text-2xl font-semibold text-green-600">{stats?.errors_today ?? 0}</div>
              <p className="mt-1 text-sm text-slate-600">Mức lỗi thấp nên giữ màu xanh thay vì làm quá căng thẳng.</p>
            </div>
            <div className="border px-4 py-3">
              <div className="text-xs font-semibold uppercase tracking-[0.16em] text-slate-500">Traffic</div>
              <div className="mt-2 text-2xl font-semibold text-slate-950">{stats?.requests_today ?? 0}</div>
              <p className="mt-1 text-sm text-slate-600">Theo dõi thêm ở trang Usage và Health khi cần drill down.</p>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
