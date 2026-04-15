'use client';

import { useEffect, useRef, useState } from 'react';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { AdminTablePagination } from '@/components/admin/admin-table-pagination';
import { AdminTableToolbar } from '@/components/admin/admin-table-toolbar';
import { getUsageLogs, type UsageLogEntry } from '@/lib/api';
import { useDebouncedValue } from '@/lib/use-debounced-value';

const statusOptions = ['all', 'success', 'rate_limited', 'cf_blocked', 'service_unavailable', 'error'];
const pageSizeOptions = [10, 20, 50, 100];

export default function UsagePage() {
  const [logs, setLogs] = useState<UsageLogEntry[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(true);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [search, setSearch] = useState('');
  const [status, setStatus] = useState('all');
  const [model, setModel] = useState('all');
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const debouncedSearch = useDebouncedValue(search, 250);
  const requestIdRef = useRef(0);

  useEffect(() => {
    setPage(1);
  }, [debouncedSearch, status, model]);

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
        });
        if (requestId !== requestIdRef.current) return;
        setLogs(data.logs);
        setTotal(data.total);
      } catch (error) {
        if (requestId !== requestIdRef.current) return;
        setErrorMessage(error instanceof Error ? error.message : 'Failed to load usage logs.');
      } finally {
        if (requestId !== requestIdRef.current) return;
        setLoading(false);
      }
    }

    void loadLogs();
  }, [debouncedSearch, model, page, pageSize, status]);

  const getStatusBadge = (value: string | null) => {
    switch (value) {
      case 'success':
        return <Badge variant="default">Success</Badge>;
      case 'rate_limited':
        return <Badge variant="secondary">Rate limited</Badge>;
      case 'cf_blocked':
        return <Badge variant="destructive">CF blocked</Badge>;
      default:
        return <Badge variant="outline">{value || 'Unknown'}</Badge>;
    }
  };

  const modelOptions = Array.from(
    new Set(['all', model, ...(logs.map((log) => log.model).filter(Boolean) as string[])])
  );

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
        <div>
          <h1 className="text-3xl font-bold">Usage Logs</h1>
          <p className="mt-1 text-sm text-slate-500">Track requests, latency, and failures across all API traffic.</p>
        </div>
        <Badge variant="outline">Total: {total} requests</Badge>
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
                    <TableHead>Account</TableHead>
                    <TableHead>API Key</TableHead>
                    <TableHead>Status</TableHead>
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
                      <TableCell>{log.account_id ? `#${log.account_id}` : '-'}</TableCell>
                      <TableCell>{log.api_key_id ? `#${log.api_key_id}` : '-'}</TableCell>
                      <TableCell>{getStatusBadge(log.status)}</TableCell>
                      <TableCell>{log.latency_ms ? `${log.latency_ms}ms` : '-'}</TableCell>
                    </TableRow>
                  )) : (
                    <TableRow>
                      <TableCell colSpan={6} className="py-8 text-center text-sm text-slate-500">
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
