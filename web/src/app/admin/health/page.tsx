'use client';

import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { adminFetch } from '@/lib/api';
import {
  Server,
  Activity,
  Users,
  Key,
  CheckCircle,
  AlertCircle,
  XCircle,
  TrendingUp,
} from 'lucide-react';

interface HealthOverview {
  total_accounts: number;
  active_accounts: number;
  total_proxies: number;
  active_proxies: number;
  total_requests_today: number;
  total_requests_week: number;
  error_rate_percent: number;
  active_users_24h: number;
  api_key_count: number;
}

export default function HealthPage() {
  const [health, setHealth] = useState<HealthOverview | null>(null);
  const [loading, setLoading] = useState(true);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  useEffect(() => {
    loadHealth();
  }, []);

  const loadHealth = async () => {
    try {
      setErrorMessage(null);
      const data = await adminFetch<HealthOverview>('/admin/health');
      setHealth(data);
    } catch (error) {
      setErrorMessage(error instanceof Error ? error.message : 'Failed to load health.');
    } finally {
      setLoading(false);
    }
  };

  const getErrorRateColor = (rate: number) => {
    if (rate < 1) return 'text-green-500';
    if (rate < 5) return 'text-amber-500';
    return 'text-red-500';
  };

  const getErrorRateBadge = (rate: number) => {
    if (rate < 1) return 'default';
    if (rate < 5) return 'secondary';
    return 'destructive';
  };

  return (
    <div className="space-y-6">
      <h1 className="text-3xl font-bold">System Health</h1>

      {errorMessage && (
        <div className="border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
          {errorMessage}
        </div>
      )}

      {loading && !health && !errorMessage && (
        <div className="text-sm text-slate-500">Loading health data...</div>
      )}

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Grok Accounts</CardTitle>
            <Server className="h-4 w-4 text-slate-400" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{health?.active_accounts || 0}</div>
            <p className="text-xs text-slate-400">
              of {health?.total_accounts || 0} total
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Active Proxies</CardTitle>
            <Server className="h-4 w-4 text-slate-400" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{health?.active_proxies || 0}</div>
            <p className="text-xs text-slate-400">
              of {health?.total_proxies || 0} total
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">API Keys</CardTitle>
            <Key className="h-4 w-4 text-slate-400" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{health?.api_key_count || 0}</div>
            <p className="text-xs text-slate-400">Active keys</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Requests Today</CardTitle>
            <Activity className="h-4 w-4 text-slate-400" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{health?.total_requests_today?.toLocaleString() || 0}</div>
            <p className="text-xs text-slate-400">24h period</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Requests This Week</CardTitle>
            <TrendingUp className="h-4 w-4 text-slate-400" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{health?.total_requests_week?.toLocaleString() || 0}</div>
            <p className="text-xs text-slate-400">7 days</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Active Users</CardTitle>
            <Users className="h-4 w-4 text-slate-400" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{health?.active_users_24h || 0}</div>
            <p className="text-xs text-slate-400">Last 24 hours</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Error Rate</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-between">
            <div>
              <div className="text-4xl font-bold">
                <span className={getErrorRateColor(health?.error_rate_percent || 0)}>
                  {(health?.error_rate_percent || 0).toFixed(2)}%
                </span>
              </div>
              <p className="text-sm text-slate-400 mt-2">
                Error rate (last 24 hours)
              </p>
            </div>
            <div className="flex gap-2">
              <Badge variant={getErrorRateBadge(health?.error_rate_percent || 0)} className="text-lg px-4 py-2">
                {health?.error_rate_percent && health.error_rate_percent < 1 ? (
                  <><CheckCircle className="h-5 w-5 mr-2" />Healthy</>
                ) : health?.error_rate_percent && health.error_rate_percent < 5 ? (
                  <><AlertCircle className="h-5 w-5 mr-2" />Warning</>
                ) : (
                  <><XCircle className="h-5 w-5 mr-2" />Critical</>
                )}
              </Badge>
            </div>
          </div>
        </CardContent>
      </Card>

      <div className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Account Health</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex justify-between items-center">
              <span className="text-slate-400">Total Accounts</span>
              <span className="font-medium">{health?.total_accounts || 0}</span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-slate-400">Active Accounts</span>
              <Badge variant="default">{health?.active_accounts || 0}</Badge>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-slate-400">Inactive Accounts</span>
              <Badge variant="secondary">
                {(health?.total_accounts || 0) - (health?.active_accounts || 0)}
              </Badge>
            </div>
            <div className="mt-4">
              <div className="h-2 bg-slate-800 rounded-full overflow-hidden">
                <div
                  className="h-full bg-green-500 transition-all"
                  style={{
                    width: `${
                      health?.total_accounts
                        ? ((health.active_accounts / health.total_accounts) * 100).toFixed(0)
                        : 0
                    }%`,
                  }}
                />
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Proxy Health</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex justify-between items-center">
              <span className="text-slate-400">Total Proxies</span>
              <span className="font-medium">{health?.total_proxies || 0}</span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-slate-400">Active Proxies</span>
              <Badge variant="default">{health?.active_proxies || 0}</Badge>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-slate-400">Inactive Proxies</span>
              <Badge variant="secondary">
                {(health?.total_proxies || 0) - (health?.active_proxies || 0)}
              </Badge>
            </div>
            <div className="mt-4">
              <div className="h-2 bg-slate-800 rounded-full overflow-hidden">
                <div
                  className="h-full bg-blue-500 transition-all"
                  style={{
                    width: `${
                      health?.total_proxies
                        ? ((health.active_proxies / health.total_proxies) * 100).toFixed(0)
                        : 0
                    }%`,
                  }}
                />
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
