'use client';

import { useEffect, useMemo, useState } from 'react';
import { AdminTablePagination } from '@/components/admin/admin-table-pagination';
import { AdminTableToolbar } from '@/components/admin/admin-table-toolbar';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { ConfirmActionDialog } from '@/components/confirm-action-dialog';
import { Badge } from '@/components/ui/badge';
import { adminFetch, listApiKeys, createApiKey, deleteApiKey } from '@/lib/api';
import { Plus, Trash2, Copy } from 'lucide-react';

interface ApiKey {
  id: number;
  key: string;
  label: string | null;
  active: boolean;
  quota_per_day: number | null;
  daily_credit_limit: number | null;
  monthly_credit_limit: number | null;
  max_input_tokens: number | null;
  max_output_tokens: number | null;
  plan_id: number | null;
  plan_name: string | null;
  created_at: string | null;
}

interface PlanOption {
  id: number;
  name: string;
  slug: string;
  active: boolean;
}

export default function ApiKeysPage() {
  const [keys, setKeys] = useState<ApiKey[]>([]);
  const [loading, setLoading] = useState(true);
  const [pageError, setPageError] = useState<string | null>(null);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [newKey, setNewKey] = useState<string | null>(null);
  const [label, setLabel] = useState('');
  const [quota, setQuota] = useState('');
  const [dailyCreditLimit, setDailyCreditLimit] = useState('');
  const [monthlyCreditLimit, setMonthlyCreditLimit] = useState('');
  const [maxInputTokens, setMaxInputTokens] = useState('');
  const [maxOutputTokens, setMaxOutputTokens] = useState('');
  const [planId, setPlanId] = useState('none');
  const [error, setError] = useState('');
  const [revealedKey, setRevealedKey] = useState<number | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<ApiKey | null>(null);
  const [deletePending, setDeletePending] = useState(false);
  const [plans, setPlans] = useState<PlanOption[]>([]);
  const [search, setSearch] = useState('');
  const [statusFilter, setStatusFilter] = useState('all');
  const [planFilter, setPlanFilter] = useState('all');
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);

  useEffect(() => {
    loadData();
  }, []);

  useEffect(() => {
    setPage(1);
  }, [search, statusFilter, planFilter]);

  const loadData = async () => {
    try {
      setPageError(null);
      const [keysData, plansData] = await Promise.all([
        listApiKeys(),
        adminFetch<PlanOption[]>('/admin/plans'),
      ]);
      setKeys(keysData);
      setPlans(plansData.filter((plan) => plan.active));
    } catch (error) {
      setPageError(error instanceof Error ? error.message : 'Failed to load API keys.');
    } finally {
      setLoading(false);
    }
  };

  const validateLabel = (value: string): boolean => {
    if (!value) return true; // Optional field
    if (value.length > 100) return false;
    return /^[a-zA-Z0-9 _-]+$/.test(value);
  };

  const validateQuota = (value: string): boolean => {
    if (!value) return true; // Optional field
    const num = parseInt(value, 10);
    return !isNaN(num) && num > 0 && num <= 10000000;
  };

  const handleCreate = async () => {
    setError('');

    if (!validateLabel(label)) {
      setError('Invalid label. Use only letters, numbers, spaces, underscores, hyphens (max 100 chars)');
      return;
    }

    if (!validateQuota(quota)) {
      setError('Invalid quota. Must be a positive number between 1 and 10,000,000');
      return;
    }

    try {
      const result = await createApiKey({
        label: label || undefined,
        quotaPerDay: quota ? parseInt(quota, 10) : undefined,
        dailyCreditLimit: dailyCreditLimit ? parseInt(dailyCreditLimit, 10) : undefined,
        monthlyCreditLimit: monthlyCreditLimit ? parseInt(monthlyCreditLimit, 10) : undefined,
        maxInputTokens: maxInputTokens ? parseInt(maxInputTokens, 10) : undefined,
        maxOutputTokens: maxOutputTokens ? parseInt(maxOutputTokens, 10) : undefined,
        planId: planId === 'none' ? undefined : parseInt(planId, 10),
      });
      setPageError(null);
      setNewKey(result.key);
      setLabel('');
      setQuota('');
      setDailyCreditLimit('');
      setMonthlyCreditLimit('');
      setMaxInputTokens('');
      setMaxOutputTokens('');
      setPlanId('none');
      loadData();
    } catch (error) {
      setError(error instanceof Error ? error.message : 'Failed to create API key.');
    }
  };

  const handleDelete = async () => {
    if (!deleteTarget) return;
    setDeletePending(true);
    try {
      await deleteApiKey(deleteTarget.id);
      setDeleteTarget(null);
      setPageError(null);
      await loadData();
    } catch (error) {
      setPageError(error instanceof Error ? error.message : 'Failed to delete API key.');
    } finally {
      setDeletePending(false);
    }
  };

  const copyToClipboard = async (text: string) => {
    try {
      if (navigator.clipboard?.writeText) {
        await navigator.clipboard.writeText(text);
      } else {
        const textArea = document.createElement('textarea');
        textArea.value = text;
        textArea.style.position = 'fixed';
        textArea.style.left = '-999999px';
        document.body.appendChild(textArea);
        textArea.select();
        document.execCommand('copy');
        document.body.removeChild(textArea);
      }
    } catch (err) {
      console.error('Failed to copy to clipboard:', err);
    }
  };

  const maskApiKey = (key: string): string => {
    if (key.length <= 8) return '*'.repeat(key.length);
    return `${key.slice(0, 4)}...${key.slice(-4)}`;
  };

  const filteredKeys = useMemo(() => {
    const keyword = search.trim().toLowerCase();
    return keys.filter((key) => {
      if (statusFilter === 'active' && !key.active) return false;
      if (statusFilter === 'revoked' && key.active) return false;
      if (planFilter !== 'all' && String(key.plan_id || 'none') !== planFilter) return false;
      if (!keyword) return true;
      return (
        (key.label || '').toLowerCase().includes(keyword) ||
        key.key.toLowerCase().includes(keyword) ||
        (key.plan_name || '').toLowerCase().includes(keyword)
      );
    });
  }, [keys, planFilter, search, statusFilter]);

  const paginatedKeys = useMemo(() => {
    const start = (page - 1) * pageSize;
    return filteredKeys.slice(start, start + pageSize);
  }, [filteredKeys, page, pageSize]);

  if (loading) {
    return <div className="text-slate-400">Loading...</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h1 className="text-3xl font-bold">API Keys</h1>
        <Button onClick={() => setDialogOpen(true)}>
          <Plus className="w-4 h-4 mr-2" />
          Create Key
        </Button>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>API Key List</CardTitle>
          <CardDescription>
            Manage API keys for client authentication
          </CardDescription>
        </CardHeader>
        <CardContent>
          <AdminTableToolbar
            searchValue={search}
            onSearchChange={setSearch}
            searchPlaceholder="Search label, key, or plan"
            summary={`${filteredKeys.length} keys`}
            filters={(
              <>
                <Select value={statusFilter} onValueChange={(value) => setStatusFilter(value || 'all')}>
                  <SelectTrigger className="w-40">
                    <SelectValue placeholder="Filter by status" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">All statuses</SelectItem>
                    <SelectItem value="active">Active</SelectItem>
                    <SelectItem value="revoked">Revoked</SelectItem>
                  </SelectContent>
                </Select>
                <Select value={planFilter} onValueChange={(value) => setPlanFilter(value || 'all')}>
                  <SelectTrigger className="w-44">
                    <SelectValue placeholder="Filter by plan" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">All plans</SelectItem>
                    <SelectItem value="none">No plan</SelectItem>
                    {plans.map((plan) => (
                      <SelectItem key={plan.id} value={String(plan.id)}>
                        {plan.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </>
            )}
          />
          {pageError && (
            <div className="mb-4 border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
              {pageError}
            </div>
          )}
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Label</TableHead>
                <TableHead>Key</TableHead>
                <TableHead>Active</TableHead>
                <TableHead>Plan</TableHead>
                <TableHead>Req/Day</TableHead>
                <TableHead>Credits</TableHead>
                <TableHead>Token Guard</TableHead>
                <TableHead>Created</TableHead>
                <TableHead className="w-32">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {paginatedKeys.length ? paginatedKeys.map((key) => (
                <TableRow key={key.id}>
                  <TableCell>
                    {key.label || <span className="text-slate-500">-</span>}
                  </TableCell>
                  <TableCell className="font-mono text-sm">
                    <div className="flex items-center gap-2">
                      {revealedKey === key.id ? (
                        <span className="text-emerald-400">{key.key}</span>
                      ) : (
                        <span className="text-slate-400">{maskApiKey(key.key)}</span>
                      )}
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => copyToClipboard(key.key)}
                        title="Copy key"
                      >
                        <Copy className="w-4 h-4" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => setRevealedKey(revealedKey === key.id ? null : key.id)}
                        title={revealedKey === key.id ? 'Hide' : 'Reveal'}
                      >
                        {revealedKey === key.id ? 'Hide' : 'Show'}
                      </Button>
                    </div>
                  </TableCell>
                  <TableCell>
                    <Badge variant={key.active ? 'default' : 'secondary'}>
                      {key.active ? 'Active' : 'Revoked'}
                    </Badge>
                  </TableCell>
                  <TableCell>
                    {key.plan_name ? (
                      <Badge variant="outline">{key.plan_name}</Badge>
                    ) : (
                      <span className="text-slate-500">No plan</span>
                    )}
                  </TableCell>
                  <TableCell>{key.quota_per_day || 'Unlimited'}</TableCell>
                  <TableCell className="text-xs text-slate-500">
                    <div>Day: {key.daily_credit_limit || 'Unlimited'}</div>
                    <div>Month: {key.monthly_credit_limit || 'Unlimited'}</div>
                  </TableCell>
                  <TableCell className="text-xs text-slate-500">
                    <div>In: {key.max_input_tokens || 'None'}</div>
                    <div>Out: {key.max_output_tokens || 'None'}</div>
                  </TableCell>
                  <TableCell className="text-slate-400">
                    {key.created_at?.slice(0, 10) || '-'}
                  </TableCell>
                  <TableCell>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => setDeleteTarget(key)}
                    >
                      <Trash2 className="w-4 h-4 text-red-500" />
                    </Button>
                  </TableCell>
                </TableRow>
              )) : (
                <TableRow>
                      <TableCell colSpan={9} className="py-8 text-center text-sm text-slate-500">
                        No API keys match the current filters.
                      </TableCell>
                </TableRow>
              )}
            </TableBody>
          </Table>
          <AdminTablePagination
            page={page}
            pageSize={pageSize}
            visibleCount={paginatedKeys.length}
            totalCount={filteredKeys.length}
            onPageChange={setPage}
            onPageSizeChange={(value) => {
              setPageSize(value);
              setPage(1);
            }}
          />
        </CardContent>
      </Card>

      <Dialog open={dialogOpen} onOpenChange={(open) => {
        setDialogOpen(open);
        if (!open) setError('');
      }}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Create API Key</DialogTitle>
            <DialogDescription>
              Generate a new API key for client authentication
            </DialogDescription>
          </DialogHeader>

          {error && !newKey && (
            <div className="p-3 bg-red-500/10 border border-red-500/20 rounded text-red-400 text-sm">
              {error}
            </div>
          )}

          {newKey ? (
            <div className="space-y-4">
              <Card className="bg-green-50 border-green-200">
                <CardHeader className="pb-2">
                  <CardTitle className="text-green-800 text-sm">
                    Key Generated!
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="font-mono text-sm break-all mb-2">
                    {newKey}
                  </div>
                  <Button
                    size="sm"
                    onClick={() => copyToClipboard(newKey)}
                  >
                    <Copy className="w-4 h-4 mr-2" />
                    Copy
                  </Button>
                </CardContent>
              </Card>
              <p className="text-sm text-amber-600">
                ⚠️ This is the only time this key will be shown. Save it securely!
              </p>
            </div>
          ) : (
            <div className="space-y-4">
              <div className="space-y-2">
                <label className="text-sm font-medium">Label (optional)</label>
                <Input
                  value={label}
                  onChange={(e) => setLabel(e.target.value)}
                  placeholder="Production Client"
                  maxLength={100}
                />
                <p className="text-xs text-slate-500">
                  Letters, numbers, spaces, underscores, hyphens (max 100 chars)
                </p>
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">
                  Test Plan (optional)
                </label>
                <Select value={planId} onValueChange={(value) => setPlanId(value || 'none')}>
                  <SelectTrigger className="w-full">
                    <SelectValue placeholder="Select a plan for this admin key" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="none">No plan override</SelectItem>
                    {plans.map((plan) => (
                      <SelectItem key={plan.id} value={String(plan.id)}>
                        {plan.name} ({plan.slug})
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <p className="text-xs text-slate-500">
                  Dùng cho admin test. User key vẫn tự lấy plan từ user.
                </p>
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">
                  Daily Quota (optional)
                </label>
                <Input
                  type="number"
                  value={quota}
                  onChange={(e) => setQuota(e.target.value)}
                  placeholder="1000"
                  min={1}
                  max={10000000}
                />
                <p className="text-xs text-slate-500">
                  Leave empty for unlimited (max: 10,000,000)
                </p>
              </div>
              <div className="grid gap-4 sm:grid-cols-2">
                <div className="space-y-2">
                  <label className="text-sm font-medium">Daily Credits (optional)</label>
                  <Input type="number" value={dailyCreditLimit} onChange={(e) => setDailyCreditLimit(e.target.value)} placeholder="50000" min={1} />
                </div>
                <div className="space-y-2">
                  <label className="text-sm font-medium">Monthly Credits (optional)</label>
                  <Input type="number" value={monthlyCreditLimit} onChange={(e) => setMonthlyCreditLimit(e.target.value)} placeholder="1000000" min={1} />
                </div>
              </div>
              <div className="grid gap-4 sm:grid-cols-2">
                <div className="space-y-2">
                  <label className="text-sm font-medium">Max Input Tokens (optional)</label>
                  <Input type="number" value={maxInputTokens} onChange={(e) => setMaxInputTokens(e.target.value)} placeholder="16000" min={1} />
                </div>
                <div className="space-y-2">
                  <label className="text-sm font-medium">Max Output Tokens (optional)</label>
                  <Input type="number" value={maxOutputTokens} onChange={(e) => setMaxOutputTokens(e.target.value)} placeholder="4000" min={1} />
                </div>
              </div>
            </div>
          )}

          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => {
                setDialogOpen(false);
                setNewKey(null);
                setLabel('');
                setQuota('');
                setDailyCreditLimit('');
                setMonthlyCreditLimit('');
                setMaxInputTokens('');
                setMaxOutputTokens('');
                setPlanId('none');
                setError('');
              }}
            >
              {newKey ? 'Done' : 'Cancel'}
            </Button>
            {!newKey && (
              <Button onClick={handleCreate}>Generate Key</Button>
            )}
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <ConfirmActionDialog
        open={!!deleteTarget}
        onOpenChange={(open) => !open && setDeleteTarget(null)}
        title="Revoke API key?"
        description={`API key ${deleteTarget?.label || deleteTarget?.key.slice(0, 8) || ''} will stop working immediately.`}
        confirmLabel="Revoke Key"
        loading={deletePending}
        onConfirm={handleDelete}
      />
    </div>
  );
}
