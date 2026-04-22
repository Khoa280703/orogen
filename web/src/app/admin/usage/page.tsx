'use client';

import { useEffect, useRef, useState } from 'react';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { AdminTablePagination } from '@/components/admin/admin-table-pagination';
import { AdminTableToolbar } from '@/components/admin/admin-table-toolbar';
import {
  getUsageLogs,
  type UsageLogAggregates,
  type UsageLogBreakdownRow,
  type UsageLogEntry,
} from '@/lib/api';
import { useDebouncedValue } from '@/lib/use-debounced-value';

const pageSizeOptions = [10, 20, 50, 100];

function formatNumber(value: number | null | undefined) {
  if (value === null || value === undefined) return '-';
  return value.toLocaleString();
}

function BreakdownTable({
  title,
  description,
  rows,
}: {
  title: string;
  description: string;
  rows: UsageLogBreakdownRow[];
}) {
  return (
    <Card>
      <CardHeader className="pb-3">
        <CardTitle className="text-sm">{title}</CardTitle>
        <CardDescription>{description}</CardDescription>
      </CardHeader>
      <CardContent>
        {rows.length ? (
          <div className="space-y-2">
            {rows.map((row) => (
              <div
                key={row.label}
                className="grid grid-cols-[minmax(0,1fr)_auto_auto] items-center gap-3 rounded-lg border px-3 py-2"
              >
                <div className="min-w-0">
                  <div className="truncate font-mono text-sm font-medium">{row.label}</div>
                  <div className="text-xs text-slate-500">
                    {formatNumber(row.requests)} req · {formatNumber(row.prompt_tokens + row.completion_tokens)} tok
                  </div>
                </div>
                <div className="text-right text-xs text-slate-500">
                  <div>Prompt {formatNumber(row.prompt_tokens)}</div>
                  <div>Comp {formatNumber(row.completion_tokens)}</div>
                </div>
                <div className="text-right">
                  <div className="text-sm font-semibold">{formatNumber(row.credits_used)}</div>
                  <div className="text-[11px] text-slate-500">credits</div>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="rounded-lg border border-dashed px-4 py-6 text-sm text-slate-500">
            No data for current filters.
          </div>
        )}
      </CardContent>
    </Card>
  );
}

export default function UsagePage() {
  const [logs, setLogs] = useState<UsageLogEntry[]>([]);
  const [total, setTotal] = useState(0);
  const [filterStatuses, setFilterStatuses] = useState<string[]>([]);
  const [filterModels, setFilterModels] = useState<string[]>([]);
  const [filterProviders, setFilterProviders] = useState<string[]>([]);
  const [aggregates, setAggregates] = useState<UsageLogAggregates>({
    prompt_tokens: 0,
    completion_tokens: 0,
    cached_tokens: 0,
    credits_used: 0,
  });
  const [providerBreakdown, setProviderBreakdown] = useState<UsageLogBreakdownRow[]>([]);
  const [modelBreakdown, setModelBreakdown] = useState<UsageLogBreakdownRow[]>([]);
  const [loading, setLoading] = useState(true);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [search, setSearch] = useState('');
  const [status, setStatus] = useState('all');
  const [model, setModel] = useState('all');
  const [provider, setProvider] = useState('all');
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const debouncedSearch = useDebouncedValue(search, 250);
  const requestIdRef = useRef(0);

  useEffect(() => {
    setPage(1);
  }, [debouncedSearch, status, model, provider]);

  useEffect(() => {
    const requestId = ++requestIdRef.current;

    async function loadLogs() {
      try {
        setLoading(true);
        setErrorMessage(null);
        const data = await getUsageLogs({
          page,
          limit: pageSize,
          search: debouncedSearch,
          status,
          model,
          provider,
        });
        if (requestId !== requestIdRef.current) return;
        setLogs(data.logs);
        setTotal(data.total);
        setFilterStatuses(data.filters.statuses);
        setFilterModels(data.filters.models);
        setFilterProviders(data.filters.providers);
        setAggregates(data.aggregates);
        setProviderBreakdown(data.breakdowns.providers);
        setModelBreakdown(data.breakdowns.models);
      } catch (error) {
        if (requestId !== requestIdRef.current) return;
        setErrorMessage(error instanceof Error ? error.message : 'Failed to load usage logs.');
      } finally {
        if (requestId !== requestIdRef.current) return;
        setLoading(false);
      }
    }

    void loadLogs();
  }, [debouncedSearch, model, page, pageSize, provider, status]);

  const getStatusBadge = (value: string | null) => {
    switch (value) {
      case 'success':
        return <Badge variant="default">Success</Badge>;
      case 'rate_limited':
        return <Badge variant="secondary">Rate limited</Badge>;
      case 'cf_blocked':
        return <Badge variant="destructive">CF blocked</Badge>;
      case 'proxy_failed':
        return <Badge variant="destructive">Proxy failed</Badge>;
      case 'unauthorized':
        return <Badge variant="destructive">Unauthorized</Badge>;
      case 'service_unavailable':
        return <Badge variant="secondary">Unavailable</Badge>;
      default:
        return <Badge variant="outline">{value || 'Unknown'}</Badge>;
    }
  };

  const statusOptions = Array.from(
    new Set(['all', status, ...filterStatuses])
  );
  const modelOptions = Array.from(
    new Set(['all', model, ...filterModels])
  );
  const providerOptions = Array.from(
    new Set(['all', provider, ...filterProviders])
  );
  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
        <div>
          <h1 className="text-3xl font-bold">Usage Logs</h1>
          <p className="mt-1 text-sm text-slate-500">Track metered gateway usage, token flow, credits, latency, and failures across provider routes.</p>
        </div>
        <Badge variant="outline">Total: {total} requests</Badge>
      </div>

      <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Prompt Tokens</CardTitle>
            <CardDescription>Total after current filters</CardDescription>
          </CardHeader>
          <CardContent className="text-2xl font-semibold">{formatNumber(aggregates.prompt_tokens)}</CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Completion Tokens</CardTitle>
            <CardDescription>Total after current filters</CardDescription>
          </CardHeader>
          <CardContent className="text-2xl font-semibold">{formatNumber(aggregates.completion_tokens)}</CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Credits Used</CardTitle>
            <CardDescription>Total after current filters</CardDescription>
          </CardHeader>
          <CardContent className="text-2xl font-semibold">{formatNumber(aggregates.credits_used)}</CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Cached Tokens</CardTitle>
            <CardDescription>Total after current filters</CardDescription>
          </CardHeader>
          <CardContent className="text-2xl font-semibold">{formatNumber(aggregates.cached_tokens)}</CardContent>
        </Card>
      </div>

      <div className="grid gap-3 xl:grid-cols-2">
        <BreakdownTable
          title="Top Providers"
          description="Highest metered providers after current filters."
          rows={providerBreakdown}
        />
        <BreakdownTable
          title="Top Models"
          description="Highest metered model slugs after current filters."
          rows={modelBreakdown}
        />
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Request History</CardTitle>
          <CardDescription>Search, filter, and page through recent API requests.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <AdminTableToolbar
            searchValue={search}
            onSearchChange={setSearch}
            searchPlaceholder="Search model, status, account, or API key"
            summary={loading ? 'Loading…' : `${logs.length} rows on this page`}
            filters={(
              <>
                <Select value={status} onValueChange={(value) => setStatus(value || 'all')}>
                  <SelectTrigger className="w-44">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {statusOptions.map((option) => (
                      <SelectItem key={option} value={option}>
                        {option === 'all' ? 'All statuses' : option}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <Select value={model} onValueChange={(value) => setModel(value || 'all')}>
                  <SelectTrigger className="w-44">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {modelOptions.map((option) => (
                      <SelectItem key={option} value={option}>
                        {option === 'all' ? 'All models' : option}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <Select value={provider} onValueChange={(value) => setProvider(value || 'all')}>
                  <SelectTrigger className="w-44">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {providerOptions.map((option) => (
                      <SelectItem key={option} value={option}>
                        {option === 'all' ? 'All providers' : option}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </>
            )}
          />

          {errorMessage ? (
            <div className="border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
              {errorMessage}
            </div>
          ) : null}

          {loading ? (
            <div className="py-8 text-sm text-slate-500">Loading usage logs...</div>
          ) : (
            <>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Time</TableHead>
                    <TableHead>Model</TableHead>
                    <TableHead>Provider</TableHead>
                    <TableHead>Account</TableHead>
                    <TableHead>API Key</TableHead>
                    <TableHead>Status</TableHead>
                    <TableHead>Tokens</TableHead>
                    <TableHead>Credits</TableHead>
                    <TableHead>Latency</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {logs.length ? logs.map((log) => (
                    <TableRow key={`${log.id}-${log.created_at}`}>
                      <TableCell className="text-slate-400">
                        {log.created_at ? new Date(log.created_at).toLocaleString() : '-'}
                      </TableCell>
                      <TableCell>{log.model || '-'}</TableCell>
                      <TableCell>{log.provider_slug || '-'}</TableCell>
                      <TableCell>{log.account_id ? `#${log.account_id}` : '-'}</TableCell>
                      <TableCell>{log.api_key_id ? `#${log.api_key_id}` : '-'}</TableCell>
                      <TableCell>{getStatusBadge(log.status)}</TableCell>
                      <TableCell className="text-xs text-slate-500">
                        <div>P {formatNumber(log.prompt_tokens)}</div>
                        <div>C {formatNumber(log.completion_tokens)}</div>
                        <div>Cache {formatNumber(log.cached_tokens)}</div>
                      </TableCell>
                      <TableCell className="text-xs text-slate-500">
                        <div>{formatNumber(log.credits_used)}</div>
                        <div>{log.estimated_usage ? 'estimated' : 'settled'}</div>
                      </TableCell>
                      <TableCell>{log.latency_ms ? `${log.latency_ms}ms` : '-'}</TableCell>
                    </TableRow>
                  )) : (
                    <TableRow>
                      <TableCell colSpan={9} className="py-8 text-center text-sm text-slate-500">
                        No usage logs match the current filters.
                      </TableCell>
                    </TableRow>
                  )}
                </TableBody>
              </Table>

              <AdminTablePagination
                page={page}
                pageSize={pageSize}
                visibleCount={logs.length}
                totalCount={total}
                pageSizeOptions={pageSizeOptions}
                onPageChange={setPage}
                onPageSizeChange={(value) => {
                  setPageSize(value);
                  setPage(1);
                }}
              />
            </>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
