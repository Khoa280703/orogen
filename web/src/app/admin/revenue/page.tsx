'use client';

import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { adminFetch } from '@/lib/api';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { DollarSign, TrendingUp, Users, CreditCard } from 'lucide-react';

interface RevenueOverview {
  total_revenue: string;
  today_revenue: string;
  week_revenue: string;
  month_revenue: string;
  active_subscribers: number;
  top_users: Array<{
    user_id: number;
    email: string;
    total_spent: string;
    transaction_count: number;
  }>;
}

interface RevenueByDay {
  date: string;
  revenue: string;
  transaction_count: number;
}

interface RevenueByMethod {
  method: string;
  revenue: string;
  transaction_count: number;
}

export default function RevenuePage() {
  const [overview, setOverview] = useState<RevenueOverview | null>(null);
  const [dailyRevenue, setDailyRevenue] = useState<RevenueByDay[]>([]);
  const [methodRevenue, setMethodRevenue] = useState<RevenueByMethod[]>([]);
  const [loading, setLoading] = useState(true);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  useEffect(() => {
    loadRevenue();
  }, []);

  const loadRevenue = async () => {
    try {
      setErrorMessage(null);
      const [overviewData, dailyData, methodData] = await Promise.all([
        adminFetch<RevenueOverview>('/admin/revenue/overview'),
        adminFetch<RevenueByDay[]>('/admin/revenue/daily'),
        adminFetch<RevenueByMethod[]>('/admin/revenue/methods'),
      ]);
      setOverview(overviewData);
      setDailyRevenue(dailyData);
      setMethodRevenue(methodData);
    } catch (error) {
      setErrorMessage(error instanceof Error ? error.message : 'Failed to load revenue.');
    } finally {
      setLoading(false);
    }
  };

  const maxRevenue = Math.max(...dailyRevenue.map(d => parseFloat(d.revenue) || 0), 1);

  return (
    <div className="space-y-6">
      <h1 className="text-3xl font-bold">Revenue Dashboard</h1>

      {errorMessage && (
        <div className="border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
          {errorMessage}
        </div>
      )}

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Revenue</CardTitle>
            <DollarSign className="h-4 w-4 text-slate-400" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-green-500">
              ${overview?.total_revenue || '0'}
            </div>
            <p className="text-xs text-slate-400">All time</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Today</CardTitle>
            <TrendingUp className="h-4 w-4 text-slate-400" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">${overview?.today_revenue || '0'}</div>
            <p className="text-xs text-slate-400">24h period</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">This Week</CardTitle>
            <TrendingUp className="h-4 w-4 text-slate-400" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">${overview?.week_revenue || '0'}</div>
            <p className="text-xs text-slate-400">7 days</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Active Subscribers</CardTitle>
            <Users className="h-4 w-4 text-slate-400" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{overview?.active_subscribers || 0}</div>
            <p className="text-xs text-slate-400">With balance &gt; 0</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Revenue (Last 30 Days)</CardTitle>
        </CardHeader>
        <CardContent>
          {loading ? (
            <div className="text-center py-8 text-slate-400">Loading...</div>
          ) : (
            <div className="h-64 flex items-end justify-between gap-2">
              {dailyRevenue.map((day) => {
                const height = (parseFloat(day.revenue) / maxRevenue) * 100;
                return (
                  <div key={day.date} className="flex-1 flex flex-col items-center gap-2">
                    <div
                      className="w-full bg-green-500 rounded-t transition-all hover:bg-green-400"
                      style={{ height: `${Math.max(height, 2)}%` }}
                      title={`$${day.revenue}`}
                    />
                    <span className="text-xs text-slate-400">
                      {new Date(day.date).toLocaleDateString('en-US', { month: 'short', day: 'numeric' })}
                    </span>
                  </div>
                );
              })}
            </div>
          )}
        </CardContent>
      </Card>

      <div className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Revenue by Method</CardTitle>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Method</TableHead>
                  <TableHead>Revenue</TableHead>
                  <TableHead>Transactions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {methodRevenue.map((method) => (
                  <TableRow key={method.method}>
                    <TableCell>
                      <div className="flex items-center gap-2">
                        <CreditCard className="h-4 w-4 text-slate-400" />
                        <span className="capitalize">{method.method}</span>
                      </div>
                    </TableCell>
                    <TableCell className="font-medium text-green-500">
                      ${method.revenue}
                    </TableCell>
                    <TableCell>{method.transaction_count}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Top Customers</CardTitle>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Email</TableHead>
                  <TableHead>Total Spent</TableHead>
                  <TableHead>Transactions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {overview?.top_users.slice(0, 5).map((user) => (
                  <TableRow key={user.user_id}>
                    <TableCell className="text-sm">{user.email}</TableCell>
                    <TableCell className="font-medium text-green-500">
                      ${user.total_spent}
                    </TableCell>
                    <TableCell>{user.transaction_count}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
