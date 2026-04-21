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
  provider_verification: Array<{
    provider_slug: string;
    provider_name: string;
    expected_auth_mode: string | null;
    has_chat_adapter: boolean;
    supports_chat_streaming: boolean;
    supports_responses_api: boolean;
    active_account_count: number;
    selectable_account_count: number;
    active_public_route_count: number;
    plan_assignment_count: number;
    ready: boolean;
    warnings: string[];
  }>;
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
            <CardTitle className="text-sm font-medium">Provider Accounts</CardTitle>
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

      <Card>
        <CardHeader>
          <CardTitle>Provider Verification Gates</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {health?.provider_verification?.length ? health.provider_verification.map((provider) => (
            <div key={provider.provider_slug} className="rounded-lg border border-slate-200 p-4 dark:border-slate-800">
              <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
                <div>
                  <div className="flex items-center gap-2">
                    <h3 className="text-lg font-semibold">{provider.provider_name}</h3>
                    <Badge variant={provider.ready ? 'default' : 'secondary'}>
                      {provider.ready ? 'No active blockers' : 'Needs follow-up'}
                    </Badge>
                  </div>
                  <p className="mt-1 text-sm text-slate-400">
                    `{provider.provider_slug}` · expected auth: {provider.expected_auth_mode || 'n/a'}
                  </p>
                </div>
                <div className="flex flex-wrap gap-2">
                  <Badge variant={provider.has_chat_adapter ? 'default' : 'destructive'}>
                    chat adapter: {provider.has_chat_adapter ? 'yes' : 'no'}
                  </Badge>
                  <Badge variant={provider.supports_chat_streaming ? 'default' : 'secondary'}>
                    streaming: {provider.supports_chat_streaming ? 'yes' : 'no'}
                  </Badge>
                  <Badge variant={provider.supports_responses_api ? 'default' : 'secondary'}>
                    responses: {provider.supports_responses_api ? 'yes' : 'no'}
                  </Badge>
                </div>
              </div>

              <div className="mt-4 grid gap-3 md:grid-cols-3">
                <div className="rounded-md bg-slate-50 p-3 dark:bg-slate-900">
                  <div className="text-xs uppercase tracking-wide text-slate-400">Selectable Accounts</div>
                  <div className="mt-1 text-xl font-semibold">{provider.selectable_account_count}</div>
                  <div className="mt-1 text-xs text-slate-400">{provider.active_account_count} active rows configured</div>
                </div>
                <div className="rounded-md bg-slate-50 p-3 dark:bg-slate-900">
                  <div className="text-xs uppercase tracking-wide text-slate-400">Public Routes</div>
                  <div className="mt-1 text-xl font-semibold">{provider.active_public_route_count}</div>
                </div>
                <div className="rounded-md bg-slate-50 p-3 dark:bg-slate-900">
                  <div className="text-xs uppercase tracking-wide text-slate-400">Selling Plans</div>
                  <div className="mt-1 text-xl font-semibold">{provider.plan_assignment_count}</div>
                </div>
              </div>

              <div className="mt-4">
                <div className="text-sm font-medium text-slate-500">Warnings</div>
                {provider.warnings.length ? (
                  <ul className="mt-2 space-y-2 text-sm text-amber-600 dark:text-amber-400">
                    {provider.warnings.map((warning) => (
                      <li key={warning} className="rounded-md border border-amber-200 bg-amber-50 px-3 py-2 dark:border-amber-900/60 dark:bg-amber-950/20">
                        {warning}
                      </li>
                    ))}
                  </ul>
                ) : (
                  <p className="mt-2 text-sm text-green-600 dark:text-green-400">
                    No provider-gate warnings for this provider.
                  </p>
                )}
              </div>
            </div>
          )) : (
            <div className="text-sm text-slate-500">No active providers found.</div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
