'use client';

import { useEffect, useMemo, useState } from 'react';
import { AdminTablePagination } from '@/components/admin/admin-table-pagination';
import { AdminTableToolbar } from '@/components/admin/admin-table-toolbar';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  listAccounts,
  createAccount,
  updateAccount,
  deleteAccount,
  listProxies,
  openAccountLoginBrowser,
  syncAccountProfile,
  type AccountCookiesInput,
  type AccountSummary,
  type ProxySummary,
} from '@/lib/api';
import { Plus, Edit, RefreshCw, Trash2 } from 'lucide-react';

export default function AccountsPage() {
  const [accounts, setAccounts] = useState<AccountSummary[]>([]);
  const [proxies, setProxies] = useState<ProxySummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [pageError, setPageError] = useState<string | null>(null);
  const [pageNotice, setPageNotice] = useState<string | null>(null);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editAccount, setEditAccount] = useState<AccountSummary | null>(null);
  const [name, setName] = useState('');
  const [cookies, setCookies] = useState('');
  const [active, setActive] = useState(true);
  const [selectedProxyId, setSelectedProxyId] = useState('none');
  const [error, setError] = useState('');
  const [deleteTarget, setDeleteTarget] = useState<AccountSummary | null>(null);
  const [deletePending, setDeletePending] = useState(false);
  const [sessionActionPending, setSessionActionPending] = useState<'launch' | 'sync' | null>(null);
  const [search, setSearch] = useState('');
  const [statusFilter, setStatusFilter] = useState('all');
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);

  useEffect(() => {
    loadData();
  }, []);

  useEffect(() => {
    setPage(1);
  }, [search, statusFilter]);

  const loadData = async () => {
    try {
      setPageError(null);
      setPageNotice(null);
      const [accountsData, proxiesData] = await Promise.all([
        listAccounts(),
        listProxies(),
      ]);
      setAccounts(accountsData);
      setProxies(proxiesData.filter((proxy) => proxy.active));
    } catch (error) {
      setPageError(error instanceof Error ? error.message : 'Failed to load account data.');
    } finally {
      setLoading(false);
    }
  };

  const resetForm = () => {
    setEditAccount(null);
    setName('');
    setCookies('');
    setActive(true);
    setSelectedProxyId('none');
    setError('');
    setSessionActionPending(null);
  };

  const parseProxyId = () => {
    if (selectedProxyId === 'none') return null;
    return Number(selectedProxyId);
  };

  const getProxyLabel = (account: AccountSummary) => {
    if (!account.proxy_id) return 'Direct';
    const proxy = proxies.find((item) => item.id === account.proxy_id);
    if (!proxy) return `Proxy #${account.proxy_id}`;
    return proxy.label || proxy.url;
  };

  const validateName = (value: string): boolean => {
    if (!value.trim()) return false;
    if (value.length > 100) return false;
    return /^[a-zA-Z0-9_-]+$/.test(value);
  };

  const validateCookies = (value: string): { valid: boolean; obj?: object } => {
    try {
      const obj = JSON.parse(value);
      if (typeof obj !== 'object' || obj === null) return { valid: false };
      if (!obj.sso || typeof obj.sso !== 'string') return { valid: false };
      return { valid: true, obj };
    } catch {
      return { valid: false };
    }
  };

  const parseCookiesInput = (
    value: string
  ): { valid: boolean; payload?: AccountCookiesInput; error?: string } => {
    const trimmed = value.trim();
    if (!trimmed) {
      return { valid: true };
    }

    if (trimmed.startsWith('{')) {
      const result = validateCookies(trimmed);
      if (!result.valid) {
        return { valid: false, error: 'Invalid cookies JSON. Must contain "sso" field.' };
      }
      return { valid: true, payload: result.obj as AccountCookiesInput };
    }

    if (!trimmed.includes('sso=')) {
      return { valid: false, error: 'Raw cookie string must contain "sso=".' };
    }

    return { valid: true, payload: trimmed };
  };

  const handleCreate = async () => {
    setError('');

    if (!validateName(name)) {
      setError('Invalid name. Use only letters, numbers, underscores, hyphens (max 100 chars).');
      return;
    }

    const cookiesResult = parseCookiesInput(cookies);
    if (!cookiesResult.valid) {
      setError(cookiesResult.error || 'Invalid cookies input.');
      return;
    }

    try {
      await createAccount({
        name,
        cookies: cookiesResult.payload,
        proxyId: parseProxyId(),
      });
      setPageError(null);
      setPageNotice(null);
      resetForm();
      setDialogOpen(false);
      await loadData();
    } catch (error) {
      setError(error instanceof Error ? error.message : 'Failed to create account.');
    }
  };

  const handleUpdate = async () => {
    setError('');

    if (!editAccount) return;

    const cookiesResult = parseCookiesInput(cookies);
    if (!cookiesResult.valid) {
      setError(cookiesResult.error || 'Invalid cookies input.');
      return;
    }

    try {
      await updateAccount(editAccount.id, {
        cookies: cookiesResult.payload,
        active,
        proxyId: parseProxyId(),
      });
      setPageError(null);
      setPageNotice(null);
      resetForm();
      setDialogOpen(false);
      await loadData();
    } catch (error) {
      setError(error instanceof Error ? error.message : 'Failed to update account.');
    }
  };

  const handleDelete = async () => {
    if (!deleteTarget) return;
    setDeletePending(true);
    try {
      await deleteAccount(deleteTarget.id);
      setDeleteTarget(null);
      setPageError(null);
      setPageNotice(null);
      await loadData();
    } catch (error) {
      setPageError(error instanceof Error ? error.message : 'Failed to delete account.');
    } finally {
      setDeletePending(false);
    }
  };

  const openEditDialog = (account: AccountSummary) => {
    setEditAccount(account);
    setName(account.name);
    const cookieRecord = account.cookies as Record<string, unknown>;
    setCookies(
      typeof cookieRecord._raw === 'string'
        ? cookieRecord._raw
        : JSON.stringify(account.cookies, null, 2)
    );
    setActive(account.active);
    setSelectedProxyId(account.proxy_id ? String(account.proxy_id) : 'none');
    setError('');
    setDialogOpen(true);
  };

  const openCreateDialog = () => {
    resetForm();
    setDialogOpen(true);
  };

  const getHealthColor = (failCount: number) => {
    if (failCount === 0) return 'bg-green-500';
    if (failCount < 3) return 'bg-yellow-500';
    return 'bg-red-500';
  };

  const getSessionTone = (status: string) => {
    switch (status) {
      case 'healthy':
        return 'border-emerald-500/20 bg-emerald-500/10 text-emerald-300';
      case 'expired':
        return 'border-red-500/20 bg-red-500/10 text-red-300';
      case 'needs_login':
        return 'border-amber-500/20 bg-amber-500/10 text-amber-300';
      case 'sync_error':
        return 'border-orange-500/20 bg-orange-500/10 text-orange-300';
      default:
        return 'border-slate-500/20 bg-slate-500/10 text-slate-300';
    }
  };

  const formatSessionStatus = (status: string) => {
    switch (status) {
      case 'healthy':
        return 'Healthy';
      case 'expired':
        return 'Expired';
      case 'needs_login':
        return 'Needs Login';
      case 'sync_error':
        return 'Sync Error';
      default:
        return 'Unknown';
    }
  };

  const handleOpenLogin = async () => {
    if (!editAccount) return;
    setError('');
    setSessionActionPending('launch');
    try {
      const result = await openAccountLoginBrowser(editAccount.id);
      await loadData();
      if (result.message) {
        setPageNotice(result.message);
      }
    } catch (actionError) {
      setError(actionError instanceof Error ? actionError.message : 'Failed to launch browser.');
    } finally {
      setSessionActionPending(null);
    }
  };

  const handleSyncProfile = async () => {
    if (!editAccount) return;
    setError('');
    setSessionActionPending('sync');
    try {
      const result = await syncAccountProfile(editAccount.id);
      await loadData();
      if (result.message) {
        setPageNotice(result.message);
      }
    } catch (actionError) {
      setError(actionError instanceof Error ? actionError.message : 'Failed to sync profile.');
    } finally {
      setSessionActionPending(null);
    }
  };

  const filteredAccounts = useMemo(() => {
    const keyword = search.trim().toLowerCase();
    return accounts.filter((account) => {
      if (statusFilter === 'active' && !account.active) return false;
      if (statusFilter === 'paused' && account.active) return false;
      if (!keyword) return true;
      const proxyLabel = getProxyLabel(account).toLowerCase();
      return account.name.toLowerCase().includes(keyword) || proxyLabel.includes(keyword);
    });
  }, [accounts, search, statusFilter, proxies]);

  const paginatedAccounts = useMemo(() => {
    const start = (page - 1) * pageSize;
    return filteredAccounts.slice(start, start + pageSize);
  }, [filteredAccounts, page, pageSize]);

  if (loading) {
    return <div className="text-slate-400">Loading...</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h1 className="text-3xl font-bold">Accounts</h1>
        <Button onClick={openCreateDialog}>
          <Plus className="w-4 h-4 mr-2" />
          Add Account
        </Button>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Account List</CardTitle>
          <CardDescription>
            Manage Grok accounts with their cookies and proxy assignments
          </CardDescription>
        </CardHeader>
        <CardContent>
          <AdminTableToolbar
            searchValue={search}
            onSearchChange={setSearch}
            searchPlaceholder="Search account or proxy"
            summary={`${filteredAccounts.length} accounts`}
            actions={(
              <Select value={statusFilter} onValueChange={(value) => setStatusFilter(value || 'all')}>
                <SelectTrigger className="w-40">
                  <SelectValue placeholder="Filter by status" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All statuses</SelectItem>
                  <SelectItem value="active">Active</SelectItem>
                  <SelectItem value="paused">Paused</SelectItem>
                </SelectContent>
              </Select>
            )}
          />
          {pageError && (
            <div className="mb-4 border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
              {pageError}
            </div>
          )}
          {pageNotice && (
            <div className="mb-4 border border-blue-200 bg-blue-50 px-4 py-3 text-sm text-blue-700">
              {pageNotice}
            </div>
          )}
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead>Health</TableHead>
                <TableHead>Session</TableHead>
                <TableHead>Proxy</TableHead>
                <TableHead>Requests</TableHead>
                <TableHead>Success</TableHead>
                <TableHead>Fail</TableHead>
                <TableHead>Last Used</TableHead>
                <TableHead className="w-32">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {paginatedAccounts.length ? paginatedAccounts.map((account) => (
                <TableRow key={account.id}>
                  <TableCell>
                    <div className="flex items-center gap-2">
                      <div
                        className={`w-3 h-3 rounded-full ${getHealthColor(account.fail_count)}`}
                      />
                      <span className="font-medium">{account.name}</span>
                      {!account.active && (
                        <Badge variant="secondary" className="ml-2">
                          Paused
                        </Badge>
                      )}
                    </div>
                  </TableCell>
                  <TableCell>
                    <Badge variant={account.active ? 'default' : 'secondary'}>
                      {account.active ? 'Active' : 'Paused'}
                    </Badge>
                  </TableCell>
                  <TableCell>
                    <div className="space-y-1">
                      <Badge
                        variant="outline"
                        className={`border ${getSessionTone(account.session_status)}`}
                      >
                        {formatSessionStatus(account.session_status)}
                      </Badge>
                      {account.session_error && (
                        <p className="max-w-[16rem] truncate text-xs text-slate-500">
                          {account.session_error}
                        </p>
                      )}
                    </div>
                  </TableCell>
                  <TableCell className="text-slate-400">
                    {getProxyLabel(account)}
                  </TableCell>
                  <TableCell>{account.request_count}</TableCell>
                  <TableCell className="text-green-500">
                    {account.success_count}
                  </TableCell>
                  <TableCell className="text-red-500">
                    {account.fail_count}
                  </TableCell>
                  <TableCell className="text-slate-400">
                    {account.last_used?.slice(0, 10) || '-'}
                  </TableCell>
                  <TableCell>
                    <div className="flex gap-2">
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => openEditDialog(account)}
                      >
                        <Edit className="w-4 h-4" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => setDeleteTarget(account)}
                      >
                        <Trash2 className="w-4 h-4 text-red-500" />
                      </Button>
                    </div>
                  </TableCell>
                </TableRow>
              )) : (
                <TableRow>
                  <TableCell colSpan={9} className="py-8 text-center text-sm text-slate-500">
                    No accounts match the current filters.
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          </Table>
          <AdminTablePagination
            page={page}
            pageSize={pageSize}
            visibleCount={paginatedAccounts.length}
            totalCount={filteredAccounts.length}
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
        <DialogContent className="max-h-[90vh] max-w-2xl overflow-y-auto">
          <DialogHeader>
            <DialogTitle>
              {editAccount ? 'Edit Account' : 'Add Account'}
            </DialogTitle>
            <DialogDescription>
              Manage cookies, browser profile, and session sync for this Grok account.
            </DialogDescription>
          </DialogHeader>
          {error && (
            <div className="p-3 bg-red-500/10 border border-red-500/20 rounded text-red-400 text-sm">
              {error}
            </div>
          )}
          <div className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">Name</label>
              <Input
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="grok-1"
                maxLength={100}
                disabled={Boolean(editAccount)}
              />
              <p className="text-xs text-slate-500">
                Letters, numbers, underscores, hyphens only (max 100 chars)
              </p>
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">Active</label>
              <div className="flex items-center gap-2">
                <Button
                  variant={active ? 'default' : 'outline'}
                  size="sm"
                  onClick={() => setActive(true)}
                >
                  Active
                </Button>
                <Button
                  variant={!active ? 'default' : 'outline'}
                  size="sm"
                  onClick={() => setActive(false)}
                >
                  Paused
                </Button>
              </div>
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">Proxy</label>
              <Select
                value={selectedProxyId}
                onValueChange={(value) => setSelectedProxyId(value ?? 'none')}
              >
                <SelectTrigger className="w-full">
                  <SelectValue placeholder="Select proxy" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="none">Direct connection</SelectItem>
                  {proxies.map((proxy) => (
                    <SelectItem key={proxy.id} value={String(proxy.id)}>
                      {proxy.label || proxy.url}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <p className="text-xs text-slate-500">
                Chon proxy cho account nay. Request cua khach van chi dung API key.
              </p>
            </div>
            {editAccount && (
              <div className="space-y-3 rounded-lg border border-slate-800 bg-slate-950/40 p-4">
                <div className="flex flex-wrap items-center gap-2">
                  <Badge variant="outline" className={`border ${getSessionTone(editAccount.session_status)}`}>
                    {formatSessionStatus(editAccount.session_status)}
                  </Badge>
                  {editAccount.cookies_synced_at && (
                    <span className="text-xs text-slate-500">
                      Last sync: {editAccount.cookies_synced_at.slice(0, 19).replace('T', ' ')}
                    </span>
                  )}
                </div>
                <p className="text-xs text-slate-500">
                  Fixed profile path: <span className="font-mono text-slate-400">{editAccount.profile_dir || `data/browser-profiles/${editAccount.name}`}</span>
                </p>
                {editAccount.session_error && (
                  <p className="text-xs text-slate-500">{editAccount.session_error}</p>
                )}
                <div className="flex flex-wrap gap-2">
                  <Button
                    type="button"
                    variant="outline"
                    onClick={handleOpenLogin}
                    disabled={sessionActionPending !== null}
                  >
                    {sessionActionPending === 'launch' && (
                      <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                    )}
                    Open Login Browser
                  </Button>
                  <Button
                    type="button"
                    variant="outline"
                    onClick={handleSyncProfile}
                    disabled={sessionActionPending !== null}
                  >
                    {sessionActionPending === 'sync' && (
                      <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                    )}
                    Sync From Profile
                  </Button>
                </div>
                <p className="text-xs text-slate-500">
                  Login once in the opened browser profile, then close that window before syncing cookies.
                </p>
              </div>
            )}
            <div className="space-y-2">
              <label className="text-sm font-medium">Cookies</label>
              <Textarea
                value={cookies}
                onChange={(e) => setCookies(e.target.value)}
                placeholder='Raw cookie string from browser or {"sso": "...", "sso-rw": "..."}'
                className="h-48 resize-y overflow-x-hidden font-mono whitespace-pre-wrap break-all [field-sizing:fixed] [overflow-wrap:anywhere]"
              />
              <p className="text-xs text-slate-500">
                Optional. If left empty, the account will rely on its fixed browser profile and you can sync later.
              </p>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDialogOpen(false)}>
              Cancel
            </Button>
            <Button onClick={editAccount ? handleUpdate : handleCreate}>
              {editAccount ? 'Save' : 'Create'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <ConfirmActionDialog
        open={!!deleteTarget}
        onOpenChange={(open) => !open && setDeleteTarget(null)}
        title="Delete account?"
        description={`Account ${deleteTarget?.name || ''} will be permanently removed.`}
        confirmLabel="Delete Account"
        loading={deletePending}
        onConfirm={handleDelete}
      />
    </div>
  );
}
