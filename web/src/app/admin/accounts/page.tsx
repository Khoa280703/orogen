'use client';

import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
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
  getAccountUsage,
  listProxies,
  openAccountLoginBrowser,
  syncAccountProfile,
  startCodexAccountLogin,
  startCodexImportLogin,
  getCodexAccountLoginStatus,
  getCodexImportLoginStatus,
  submitCodexAccountCallback,
  submitCodexImportCallback,
  refreshCodexAccountToken,
  type AccountCookiesInput,
  type AccountUsageSummary,
  type AccountSummary,
  type CodexLoginSession,
  type ProxySummary,
} from '@/lib/api';
import { Edit, Plus, RefreshCw, Trash2 } from 'lucide-react';

const PROVIDER_GROK = 'grok';
const PROVIDER_CODEX = 'codex';
const TERMINAL_CODEX_LOGIN_STATUSES = new Set(['completed', 'failed', 'expired']);

function formatCodexLoginStatus(status: string | null | undefined) {
  switch (status) {
    case 'starting':
      return 'Starting';
    case 'awaiting_user':
      return 'Waiting For Verification';
    case 'completed':
      return 'Connected';
    case 'expired':
      return 'Expired';
    case 'failed':
      return 'Failed';
    default:
      return status || 'Unknown';
  }
}

function formatUsageReset(value: string | null | undefined) {
  if (!value) {
    return '-';
  }

  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) {
    return value.replace('T', ' ').slice(0, 19);
  }

  const parts = new Intl.DateTimeFormat(undefined, {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
  }).formatToParts(parsed);

  const lookup = (type: Intl.DateTimeFormatPartTypes) =>
    parts.find((part) => part.type === type)?.value || '00';

  return `${lookup('year')}-${lookup('month')}-${lookup('day')} ${lookup('hour')}:${lookup('minute')}:${lookup('second')}`;
}

export default function AccountsPage() {
  const [accounts, setAccounts] = useState<AccountSummary[]>([]);
  const [proxies, setProxies] = useState<ProxySummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [pageError, setPageError] = useState<string | null>(null);
  const [pageNotice, setPageNotice] = useState<string | null>(null);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editAccount, setEditAccount] = useState<AccountSummary | null>(null);
  const [name, setName] = useState('');
  const [providerSlug, setProviderSlug] = useState(PROVIDER_GROK);
  const [credentials, setCredentials] = useState('');
  const [active, setActive] = useState(true);
  const [selectedProxyId, setSelectedProxyId] = useState('none');
  const [error, setError] = useState('');
  const [deleteTarget, setDeleteTarget] = useState<AccountSummary | null>(null);
  const [deletePending, setDeletePending] = useState(false);
  const [sessionActionPending, setSessionActionPending] = useState<string | null>(null);
  const [codexLoginSession, setCodexLoginSession] = useState<CodexLoginSession | null>(null);
  const [codexManualCallbackUrl, setCodexManualCallbackUrl] = useState('');
  const [search, setSearch] = useState('');
  const [statusFilter, setStatusFilter] = useState('all');
  const [activeProviderTab, setActiveProviderTab] = useState(PROVIDER_GROK);
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const [accountUsageById, setAccountUsageById] = useState<Record<number, AccountUsageSummary>>({});
  const [usageLoadingById, setUsageLoadingById] = useState<Record<number, boolean>>({});
  const [usageErrorById, setUsageErrorById] = useState<Record<number, string | null>>({});
  const accountsRef = useRef<AccountSummary[]>([]);
  const editAccountIdRef = useRef<number | null>(null);
  const codexLoginPending = codexLoginSession && !TERMINAL_CODEX_LOGIN_STATUSES.has(codexLoginSession.status);
  const credentialPreview = editAccount?.credential_preview || {};
  const codexSessionLabel = editAccount?.name || name.trim() || 'new Codex account';

  useEffect(() => {
    accountsRef.current = accounts;
    editAccountIdRef.current = editAccount?.id ?? null;
  }, [accounts, editAccount]);

  const loadData = useCallback(async () => {
    try {
      setPageError(null);
      setPageNotice(null);
      const [accountsData, proxiesData] = await Promise.all([listAccounts(), listProxies()]);
      const existingAccountsById = new Map(accountsRef.current.map((account) => [account.id, account]));
      const editAccountId = editAccountIdRef.current;
      setAccounts(accountsData);
      setAccountUsageById((current) =>
        Object.fromEntries(Object.entries(current).filter(([accountId]) => {
          const nextAccount = accountsData.find((account) => account.id === Number(accountId));
          if (!nextAccount) {
            return false;
          }

          const previousAccount = existingAccountsById.get(nextAccount.id);
          if (!previousAccount) {
            return true;
          }

          return (
            previousAccount.session_status === nextAccount.session_status &&
            previousAccount.external_account_id === nextAccount.external_account_id &&
            previousAccount.session_error === nextAccount.session_error
          );
        }))
      );
      setUsageLoadingById((current) =>
        Object.fromEntries(
          Object.entries(current).filter(([accountId]) => accountsData.some((account) => account.id === Number(accountId)))
        )
      );
      setUsageErrorById((current) =>
        Object.fromEntries(
          Object.entries(current).filter(([accountId]) => accountsData.some((account) => account.id === Number(accountId)))
        )
      );
      if (editAccountId !== null) {
        setEditAccount(accountsData.find((account) => account.id === editAccountId) || null);
      }
      setProxies(proxiesData.filter((proxy) => proxy.active));
    } catch (loadError) {
      setPageError(loadError instanceof Error ? loadError.message : 'Failed to load account data.');
    } finally {
      setLoading(false);
    }
  }, []);

  const resetForm = useCallback(() => {
    setEditAccount(null);
    setName('');
    setProviderSlug(activeProviderTab || PROVIDER_GROK);
    setCredentials('');
    setActive(true);
    setSelectedProxyId('none');
    setError('');
    setSessionActionPending(null);
    setCodexLoginSession(null);
    setCodexManualCallbackUrl('');
  }, [activeProviderTab]);

  const finalizeCodexLoginCompletion = useCallback(async (session: CodexLoginSession) => {
    setCodexLoginSession(session);
    setCodexManualCallbackUrl('');
    await loadData();

    if (editAccount) {
      setPageNotice(`Codex account ${editAccount.name} connected successfully.`);
      return;
    }

    const importedAccount = session.account_id > 0
      ? accounts.find((account) => account.id === session.account_id)
      : null;
    const importedLabel = importedAccount?.name || name.trim() || `account #${session.account_id}`;

    resetForm();
    setDialogOpen(false);
    setPageNotice(`Codex account ${importedLabel} imported successfully.`);
  }, [accounts, editAccount, loadData, name, resetForm]);

  useEffect(() => {
    void loadData();
  }, [loadData]);

  useEffect(() => {
    setPage(1);
  }, [search, statusFilter, activeProviderTab]);

  useEffect(() => {
    if (!dialogOpen || providerSlug !== PROVIDER_CODEX || !codexLoginSession) {
      return;
    }
    if (TERMINAL_CODEX_LOGIN_STATUSES.has(codexLoginSession.status)) {
      return;
    }

    const timeoutId = window.setTimeout(async () => {
      try {
        const result = editAccount
          ? await getCodexAccountLoginStatus(editAccount.id)
          : await getCodexImportLoginStatus(codexLoginSession.session_id);
        setCodexLoginSession(result.session);
        if (result.session.status === 'completed') {
          await finalizeCodexLoginCompletion(result.session);
        }
      } catch (statusError) {
        if (
          statusError instanceof Error &&
          (statusError.message.includes('No active Codex login session') ||
            statusError.message.includes('Codex login session was not found'))
        ) {
          setCodexLoginSession(null);
          return;
        }
        setError(statusError instanceof Error ? statusError.message : 'Failed to refresh Codex login status.');
      }
    }, 2000);

    return () => window.clearTimeout(timeoutId);
  }, [codexLoginSession, dialogOpen, editAccount, finalizeCodexLoginCompletion, providerSlug]);

  const refreshAccountUsage = useCallback(async (accountId: number) => {
    setUsageLoadingById((current) => ({ ...current, [accountId]: true }));
    setUsageErrorById((current) => ({ ...current, [accountId]: null }));
    try {
      const usage = await getAccountUsage(accountId);
      setAccountUsageById((current) => ({ ...current, [accountId]: usage }));
      return usage;
    } catch (usageError) {
      const message = usageError instanceof Error ? usageError.message : 'Failed to load account usage.';
      setUsageErrorById((current) => ({ ...current, [accountId]: message }));
      throw usageError;
    } finally {
      setUsageLoadingById((current) => ({ ...current, [accountId]: false }));
    }
  }, []);

  const refreshUsageForAccounts = useCallback(async (accountIds: number[]) => {
    const uniqueAccountIds = Array.from(new Set(accountIds));
    if (!uniqueAccountIds.length) {
      return;
    }

    const results = await Promise.allSettled(uniqueAccountIds.map((accountId) => refreshAccountUsage(accountId)));
    const failedCount = results.filter((result) => result.status === 'rejected').length;
    if (failedCount > 0) {
      setPageError(
        failedCount === uniqueAccountIds.length
          ? 'Failed to refresh usage for the visible accounts.'
          : `Refreshed usage with ${failedCount} account(s) failed.`
      );
    }
  }, [refreshAccountUsage]);

  const parseProxyId = () => (selectedProxyId === 'none' ? null : Number(selectedProxyId));

  const getProxyLabel = useCallback((account: AccountSummary) => {
    if (!account.proxy_id) return 'Direct';
    const proxy = proxies.find((item) => item.id === account.proxy_id);
    return proxy ? proxy.url : `Proxy #${account.proxy_id}`;
  }, [proxies]);

  const providerTabs = useMemo(() => {
    const slugs = Array.from(new Set(accounts.map((account) => account.provider_slug)));
    const preferredOrder = [PROVIDER_GROK, PROVIDER_CODEX];
    const ordered = preferredOrder.filter((slug) => slugs.includes(slug));
    const extra = slugs.filter((slug) => !preferredOrder.includes(slug)).sort((left, right) => left.localeCompare(right));
    const result = [...ordered, ...extra];
    return result.length ? result : preferredOrder;
  }, [accounts]);

  useEffect(() => {
    if (providerTabs.includes(activeProviderTab)) {
      return;
    }
    setActiveProviderTab(providerTabs[0] || PROVIDER_GROK);
  }, [activeProviderTab, providerTabs]);

  const validateName = (value: string): boolean =>
    Boolean(value.trim()) && value.length <= 100 && /^[a-zA-Z0-9_-]+$/.test(value);

  const validateGrokCredentials = (value: string): { valid: boolean; payload?: AccountCookiesInput; error?: string } => {
    const trimmed = value.trim();
    if (!trimmed) return { valid: true };

    if (trimmed.startsWith('{')) {
      try {
        const parsed = JSON.parse(trimmed);
        if (typeof parsed !== 'object' || parsed === null || typeof parsed.sso !== 'string') {
          return { valid: false, error: 'Invalid Grok cookies JSON. Must contain "sso".' };
        }
        return { valid: true, payload: parsed as AccountCookiesInput };
      } catch {
        return { valid: false, error: 'Invalid Grok cookies JSON.' };
      }
    }

    if (!trimmed.includes('sso=')) {
      return { valid: false, error: 'Raw Grok cookie string must contain "sso=".' };
    }

    return { valid: true, payload: trimmed };
  };

  const validateCodexCredentials = (value: string): { valid: boolean; payload?: AccountCookiesInput; error?: string } => {
    const trimmed = value.trim();
    if (!trimmed) return { valid: true };

    try {
      const parsed = JSON.parse(trimmed);
      if (typeof parsed !== 'object' || parsed === null || typeof (parsed as { access_token?: unknown }).access_token !== 'string') {
        return { valid: false, error: 'Invalid Codex token JSON. Must contain "access_token".' };
      }
      return { valid: true, payload: parsed as AccountCookiesInput };
    } catch {
      return { valid: false, error: 'Codex credentials must be valid JSON.' };
    }
  };

  const parseCredentialsInput = (
    provider: string,
    value: string
  ): { valid: boolean; payload?: AccountCookiesInput; error?: string } => {
    if (provider === PROVIDER_CODEX) {
      return validateCodexCredentials(value);
    }
    return validateGrokCredentials(value);
  };

  const handleCreate = async () => {
    setError('');

    if (providerSlug === PROVIDER_CODEX && !credentials.trim()) {
      setError('Với Codex, hãy bấm Start Codex Login để import account mới, hoặc dán token JSON để import nâng cao.');
      return;
    }

    if (!validateName(name)) {
      setError('Invalid name. Use only letters, numbers, underscores, hyphens (max 100 chars).');
      return;
    }

    const parsed = parseCredentialsInput(providerSlug, credentials);
    if (!parsed.valid) {
      setError(parsed.error || 'Invalid credentials input.');
      return;
    }

    try {
      await createAccount({
        name,
        providerSlug,
        credentials: parsed.payload,
        proxyId: parseProxyId(),
      });
      resetForm();
      setDialogOpen(false);
      await loadData();
    } catch (createError) {
      setError(createError instanceof Error ? createError.message : 'Failed to create account.');
    }
  };

  const handleUpdate = async () => {
    if (!editAccount) return;
    setError('');

    const parsed = parseCredentialsInput(editAccount.provider_slug, credentials);
    if (!parsed.valid) {
      setError(parsed.error || 'Invalid credentials input.');
      return;
    }

    try {
      await updateAccount(editAccount.id, {
        credentials: parsed.payload,
        active,
        proxyId: parseProxyId(),
      });
      resetForm();
      setDialogOpen(false);
      await loadData();
    } catch (updateError) {
      setError(updateError instanceof Error ? updateError.message : 'Failed to update account.');
    }
  };

  const handleDelete = async () => {
    if (!deleteTarget) return;
    setDeletePending(true);
    try {
      await deleteAccount(deleteTarget.id);
      setDeleteTarget(null);
      await loadData();
    } catch (deleteError) {
      setPageError(deleteError instanceof Error ? deleteError.message : 'Failed to delete account.');
    } finally {
      setDeletePending(false);
    }
  };

  const openEditDialog = (account: AccountSummary) => {
    setEditAccount(account);
    setName(account.name);
    setProviderSlug(account.provider_slug);
    setCredentials('');
    setActive(account.active);
    setSelectedProxyId(account.proxy_id ? String(account.proxy_id) : 'none');
    setError('');
    setCodexLoginSession(null);
    setDialogOpen(true);
  };

  const openCreateDialog = () => {
    resetForm();
    setProviderSlug(activeProviderTab || PROVIDER_GROK);
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
        return 'border-border bg-background text-foreground';
      case 'expired':
        return 'border-border bg-muted text-foreground';
      case 'needs_login':
        return 'border-border bg-muted text-foreground';
      case 'sync_error':
      case 'refresh_failed':
        return 'border-border bg-muted text-foreground';
      default:
        return 'border-border bg-muted/70 text-muted-foreground';
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
      case 'refresh_failed':
        return 'Refresh Failed';
      default:
        return 'Unknown';
    }
  };

  const getRoutingTone = (state: string) => {
    switch (state) {
      case 'healthy':
        return 'border-border bg-background text-foreground';
      case 'candidate':
        return 'border-border bg-muted text-foreground';
      case 'cooling_down':
        return 'border-border bg-muted text-foreground';
      case 'auth_invalid':
      case 'refresh_failed':
        return 'border-border bg-muted text-foreground';
      case 'paused':
        return 'border-border bg-muted text-muted-foreground';
      default:
        return 'border-border bg-muted/70 text-muted-foreground';
    }
  };

  const formatRoutingState = (state: string) => {
    switch (state) {
      case 'healthy':
        return 'Healthy';
      case 'candidate':
        return 'Candidate';
      case 'cooling_down':
        return 'Cooling';
      case 'auth_invalid':
        return 'Auth Invalid';
      case 'refresh_failed':
        return 'Refresh Failed';
      case 'paused':
        return 'Paused';
      default:
        return state || 'Unknown';
    }
  };

  const shouldShowRoutingBadge = (sessionStatus: string, routingState: string) =>
    formatSessionStatus(sessionStatus).toLowerCase() !== formatRoutingState(routingState).toLowerCase();

  const handleOpenLogin = async () => {
    if (!editAccount) return;
    setError('');
    setSessionActionPending('launch');
    try {
      const result = await openAccountLoginBrowser(editAccount.id);
      await loadData();
      if (result.message) setPageNotice(result.message);
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
      if (result.message) setPageNotice(result.message);
    } catch (actionError) {
      setError(actionError instanceof Error ? actionError.message : 'Failed to sync profile.');
    } finally {
      setSessionActionPending(null);
    }
  };

  const handleStartCodexLogin = async () => {
    setError('');
    setSessionActionPending('codex-login');
    try {
      const result = editAccount
        ? await startCodexAccountLogin(editAccount.id)
        : await startCodexImportLogin({
            name: name.trim() || undefined,
            proxyId: parseProxyId(),
          });
      setCodexLoginSession(result.session);
      setCodexManualCallbackUrl('');
      if (result.session.user_code) {
        setPageNotice(`Codex verification code generated for ${codexSessionLabel}.`);
      } else if (result.session.verification_url) {
        setPageNotice(`Codex browser login link generated for ${codexSessionLabel}.`);
      } else {
        setPageNotice(`Starting Codex login for ${codexSessionLabel}.`);
      }
    } catch (actionError) {
      setError(actionError instanceof Error ? actionError.message : 'Failed to start Codex login.');
    } finally {
      setSessionActionPending(null);
    }
  };

  const handleSubmitCodexCallback = async () => {
    const callbackUrl = codexManualCallbackUrl.trim();
    if (!callbackUrl) {
      setError('Paste the full callback URL first.');
      return;
    }

    setError('');
    setSessionActionPending('codex-callback');
    try {
      const result = editAccount
        ? await submitCodexAccountCallback(editAccount.id, callbackUrl)
        : await submitCodexImportCallback(codexLoginSession?.session_id || '', callbackUrl);
      setCodexLoginSession(result.session);
      if (result.session.status === 'completed') {
        await finalizeCodexLoginCompletion(result.session);
      } else {
        setPageNotice('Codex callback submitted. Waiting for local login session to finish.');
      }
    } catch (actionError) {
      setError(actionError instanceof Error ? actionError.message : 'Failed to submit Codex callback.');
    } finally {
      setSessionActionPending(null);
    }
  };

  const handleRefreshCodexLoginStatus = async () => {
    setError('');
    setSessionActionPending('codex-status');
    try {
      const result = editAccount
        ? await getCodexAccountLoginStatus(editAccount.id)
        : await getCodexImportLoginStatus(codexLoginSession?.session_id || '');
      setCodexLoginSession(result.session);
      if (result.session.status === 'completed') {
        await finalizeCodexLoginCompletion(result.session);
      }
    } catch (actionError) {
      setError(actionError instanceof Error ? actionError.message : 'Failed to refresh Codex login status.');
    } finally {
      setSessionActionPending(null);
    }
  };

  const handleRefreshCodexToken = async () => {
    if (!editAccount) return;
    setError('');
    setSessionActionPending('codex-refresh');
    try {
      const result = await refreshCodexAccountToken(editAccount.id);
      await loadData();
      if (result.message) setPageNotice(result.message);
    } catch (actionError) {
      await loadData();
      setError(actionError instanceof Error ? actionError.message : 'Failed to refresh Codex token.');
    } finally {
      setSessionActionPending(null);
    }
  };

  const filteredAccounts = useMemo(() => {
    const keyword = search.trim().toLowerCase();
    return accounts.filter((account) => {
      if (statusFilter === 'active' && !account.active) return false;
      if (statusFilter === 'paused' && account.active) return false;
      if (account.provider_slug !== activeProviderTab) return false;
      if (!keyword) return true;

      const accountLabel = (account.account_label || '').toLowerCase();
      const externalId = (account.external_account_id || '').toLowerCase();
      const proxyLabel = getProxyLabel(account).toLowerCase();
      return (
        account.name.toLowerCase().includes(keyword) ||
        account.provider_slug.toLowerCase().includes(keyword) ||
        accountLabel.includes(keyword) ||
        externalId.includes(keyword) ||
        proxyLabel.includes(keyword)
      );
    });
  }, [accounts, activeProviderTab, getProxyLabel, search, statusFilter]);

  const paginatedAccounts = useMemo(() => {
    const start = (page - 1) * pageSize;
    return filteredAccounts.slice(start, start + pageSize);
  }, [filteredAccounts, page, pageSize]);

  const providerStats = useMemo(() => {
    const currentProviderAccounts = accounts.filter((account) => account.provider_slug === activeProviderTab);
    const totalRequests = currentProviderAccounts.reduce((sum, account) => sum + account.request_count, 0);
    const totalSuccess = currentProviderAccounts.reduce((sum, account) => sum + account.success_count, 0);
    const totalFail = currentProviderAccounts.reduce((sum, account) => sum + account.fail_count, 0);
    const active = currentProviderAccounts.filter((account) => account.active).length;
    const healthy = currentProviderAccounts.filter(
      (account) => account.session_status === 'healthy' && account.routing_state === 'healthy'
    ).length;
    const completedAttempts = totalSuccess + totalFail;
    const successRate = completedAttempts > 0 ? Math.round((totalSuccess / completedAttempts) * 100) : 0;
    const averageRequests = currentProviderAccounts.length > 0 ? Math.round(totalRequests / currentProviderAccounts.length) : 0;

    return {
      total: currentProviderAccounts.length,
      active,
      paused: Math.max(currentProviderAccounts.length - active, 0),
      healthy,
      totalRequests,
      totalSuccess,
      totalFail,
      successRate,
      averageRequests,
    };
  }, [accounts, activeProviderTab]);

  useEffect(() => {
    if (activeProviderTab !== PROVIDER_CODEX || !paginatedAccounts.length) {
      return;
    }

    const missingUsageIds = paginatedAccounts
      .map((account) => account.id)
      .filter((accountId) => !accountUsageById[accountId] && !usageLoadingById[accountId]);

    if (!missingUsageIds.length) {
      return;
    }

    void refreshUsageForAccounts(missingUsageIds);
  }, [activeProviderTab, accountUsageById, paginatedAccounts, refreshUsageForAccounts, usageLoadingById]);

  useEffect(() => {
    if (!dialogOpen || !editAccount || editAccount.provider_slug !== PROVIDER_CODEX) {
      return;
    }

    if (accountUsageById[editAccount.id] || usageLoadingById[editAccount.id]) {
      return;
    }

    void refreshAccountUsage(editAccount.id);
  }, [accountUsageById, dialogOpen, editAccount, refreshAccountUsage, usageLoadingById]);

  const renderCodexUsage = (account: AccountSummary, mode: 'table' | 'detail' = 'table') => {
    const usage = accountUsageById[account.id];
    const isLoadingUsage = Boolean(usageLoadingById[account.id]);
    const usageError = usageErrorById[account.id];
    const sessionQuota = usage?.quotas.session;
    const weeklyQuota = usage?.quotas.weekly;

    const quotaCardClass =
      mode === 'detail'
        ? 'rounded-xl border border-border/60 bg-muted/25 p-4'
        : 'rounded-lg border border-border/60 bg-muted/20 p-3';

    if (isLoadingUsage && !usage) {
      return <p className="text-xs text-muted-foreground">Loading usage...</p>;
    }

    if (usageError && !usage) {
      return (
        <div className="space-y-2">
          <p className="max-w-[18rem] text-xs text-red-500">{usageError}</p>
          <Button variant="outline" size="sm" onClick={() => void refreshAccountUsage(account.id)}>
            Retry
          </Button>
        </div>
      );
    }

    if (!usage) {
      return (
        <Button variant="outline" size="sm" onClick={() => void refreshAccountUsage(account.id)}>
          Load usage
        </Button>
      );
    }

    if (mode === 'table') {
      return (
        <div className="max-w-full space-y-1 overflow-hidden">
          <div className="flex flex-wrap items-center gap-1">
            <Badge variant="outline" className="border-border bg-background px-1.5 py-0 text-[10px] text-foreground">
              plan {usage.plan || 'unknown'}
            </Badge>
            {usage.limit_reached !== null && (
              <Badge
                variant="outline"
                className={`border px-1.5 py-0 text-[10px] ${
                  usage.limit_reached
                    ? 'border-foreground bg-foreground text-background'
                    : 'border-border bg-muted text-foreground'
                }`}
              >
                {usage.limit_reached ? 'limit reached' : 'within limit'}
              </Badge>
            )}
            <Button
              variant="ghost"
              size="sm"
              className="h-6 w-6 shrink-0 p-0"
              onClick={() => void refreshAccountUsage(account.id)}
              disabled={isLoadingUsage}
            >
              <RefreshCw className={`h-4 w-4 ${isLoadingUsage ? 'animate-spin' : ''}`} />
            </Button>
          </div>
          <div className="space-y-0.5 text-[11px] text-muted-foreground">
            <p className="truncate">
              S: {sessionQuota ? `${sessionQuota.used}% used, ${sessionQuota.remaining}% left` : '-'}
            </p>
            <p className="truncate">
              W: {weeklyQuota ? `${weeklyQuota.used}% used, ${weeklyQuota.remaining}% left` : '-'}
            </p>
            <p className="truncate">At: {formatUsageReset(usage.fetched_at)}</p>
          </div>
        </div>
      );
    }

    return (
      <div className="space-y-4">
        <div className="flex flex-wrap items-center gap-2">
          <Badge variant="outline" className="border-border bg-background text-foreground">
            plan {usage.plan || 'unknown'}
          </Badge>
          {usage.limit_reached !== null && (
            <Badge
              variant="outline"
              className={`border ${
                usage.limit_reached
                  ? 'border-foreground bg-foreground text-background'
                  : 'border-border bg-muted text-foreground'
              }`}
            >
              {usage.limit_reached ? 'limit reached' : 'within limit'}
            </Badge>
          )}
          <Button variant="ghost" size="sm" onClick={() => void refreshAccountUsage(account.id)} disabled={isLoadingUsage}>
            <RefreshCw className={`h-4 w-4 ${isLoadingUsage ? 'animate-spin' : ''}`} />
          </Button>
        </div>
        {usage.message && (
          <p className="max-w-[20rem] text-xs text-muted-foreground">{usage.message}</p>
        )}
        <div className="grid gap-3 sm:grid-cols-2">
          <div className={quotaCardClass}>
            <p className="mb-1 text-[11px] font-medium uppercase tracking-[0.12em] text-muted-foreground">Session</p>
            <p className="text-sm font-medium text-foreground">
              {sessionQuota ? `${sessionQuota.used}% used, ${sessionQuota.remaining}% left` : '-'}
            </p>
            <p className="mt-1 text-xs text-muted-foreground">
              Reset: {formatUsageReset(sessionQuota?.reset_at)}
            </p>
          </div>
          <div className={quotaCardClass}>
            <p className="mb-1 text-[11px] font-medium uppercase tracking-[0.12em] text-muted-foreground">Weekly</p>
            <p className="text-sm font-medium text-foreground">
              {weeklyQuota ? `${weeklyQuota.used}% used, ${weeklyQuota.remaining}% left` : '-'}
            </p>
            <p className="mt-1 text-xs text-muted-foreground">
              Reset: {formatUsageReset(weeklyQuota?.reset_at)}
            </p>
          </div>
        </div>
        <div className="text-xs text-muted-foreground">
          Fetched: {formatUsageReset(usage.fetched_at)}
        </div>
      </div>
    );
  };

  if (loading) {
    return <div className="text-slate-400">Loading...</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold">Accounts</h1>
        <Button onClick={openCreateDialog}>
          <Plus className="mr-2 h-4 w-4" />
          Add Account
        </Button>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Provider Accounts</CardTitle>
          <CardDescription>
            Track pool health and request distribution for the current provider tab.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex flex-wrap gap-2">
            {providerTabs.map((provider) => (
              <Button
                key={provider}
                type="button"
                variant={provider === activeProviderTab ? 'default' : 'outline'}
                onClick={() => setActiveProviderTab(provider)}
                className="min-w-24 justify-between gap-2"
              >
                <span>{provider}</span>
                <Badge variant="secondary">{accounts.filter((account) => account.provider_slug === provider).length}</Badge>
              </Button>
            ))}
          </div>

          <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
            <Card size="sm" className="border border-border/70 bg-background/80 py-0 shadow-sm">
              <CardContent className="py-4">
                <p className="text-[11px] font-medium uppercase tracking-[0.12em] text-muted-foreground">Accounts</p>
                <p className="mt-3 text-2xl font-semibold text-foreground">{providerStats.total.toLocaleString()}</p>
                <p className="mt-1 text-xs text-muted-foreground">
                  Active {providerStats.active.toLocaleString()} · Paused {providerStats.paused.toLocaleString()}
                </p>
              </CardContent>
            </Card>
            <Card size="sm" className="border border-border/70 bg-background/80 py-0 shadow-sm">
              <CardContent className="py-4">
                <p className="text-[11px] font-medium uppercase tracking-[0.12em] text-muted-foreground">Requests</p>
                <p className="mt-3 text-2xl font-semibold text-foreground">{providerStats.totalRequests.toLocaleString()}</p>
                <p className="mt-1 text-xs text-muted-foreground">
                  Avg {providerStats.averageRequests.toLocaleString()} / account
                </p>
              </CardContent>
            </Card>
            <Card size="sm" className="border border-border/70 bg-background/80 py-0 shadow-sm">
              <CardContent className="py-4">
                <p className="text-[11px] font-medium uppercase tracking-[0.12em] text-muted-foreground">Success Rate</p>
                <p className="mt-3 text-2xl font-semibold text-foreground">{providerStats.successRate}%</p>
                <p className="mt-1 text-xs text-muted-foreground">
                  Success {providerStats.totalSuccess.toLocaleString()} · Fail {providerStats.totalFail.toLocaleString()}
                </p>
              </CardContent>
            </Card>
            <Card size="sm" className="border border-border/70 bg-background/80 py-0 shadow-sm">
              <CardContent className="py-4">
                <p className="text-[11px] font-medium uppercase tracking-[0.12em] text-muted-foreground">Healthy</p>
                <p className="mt-3 text-2xl font-semibold text-foreground">
                  {providerStats.healthy.toLocaleString()} / {providerStats.active.toLocaleString()}
                </p>
                <p className="mt-1 text-xs text-muted-foreground">Accounts ready to serve traffic right now.</p>
              </CardContent>
            </Card>
          </div>

          <AdminTableToolbar
            searchValue={search}
            onSearchChange={setSearch}
            searchPlaceholder="Search account, label, external id, or proxy"
            summary={`${filteredAccounts.length} ${activeProviderTab} account(s)`}
            actions={(
              <div className="flex gap-2">
                <Select value={statusFilter} onValueChange={(value) => setStatusFilter(value || 'all')}>
                  <SelectTrigger className="w-36">
                    <SelectValue placeholder="Status" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">All statuses</SelectItem>
                    <SelectItem value="active">Active</SelectItem>
                    <SelectItem value="paused">Paused</SelectItem>
                  </SelectContent>
                </Select>
                {activeProviderTab === PROVIDER_CODEX && (
                  <Button
                    type="button"
                    variant="outline"
                    onClick={() => void refreshUsageForAccounts(paginatedAccounts.map((account) => account.id))}
                    disabled={!paginatedAccounts.length || paginatedAccounts.every((account) => usageLoadingById[account.id])}
                  >
                    <RefreshCw className="mr-2 h-4 w-4" />
                    Refresh Usage
                  </Button>
                )}
              </div>
            )}
          />

          {pageError && (
            <div className="mb-4 rounded border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
              {pageError}
            </div>
          )}
          {pageNotice && (
            <div className="mb-4 rounded border border-blue-200 bg-blue-50 px-4 py-3 text-sm text-blue-700">
              {pageNotice}
            </div>
          )}

          <Table className="table-fixed">
            <TableHeader>
              <TableRow>
                <TableHead className="w-[18%]">Name</TableHead>
                <TableHead className="w-[18%]">Session</TableHead>
                <TableHead className="w-[16%]">Proxy</TableHead>
                <TableHead className="w-[7%]">Req</TableHead>
                <TableHead className="w-[7%]">Ok</TableHead>
                <TableHead className="w-[7%]">Fail</TableHead>
                {activeProviderTab === PROVIDER_CODEX && <TableHead className="w-[17%]">Usage</TableHead>}
                <TableHead className="w-[10%]">Last Used</TableHead>
                <TableHead className="w-[72px]">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {paginatedAccounts.length ? paginatedAccounts.map((account) => (
                <TableRow key={account.id}>
                  <TableCell className="align-top">
                    <div className="flex min-w-0 items-start gap-2">
                      <div className={`mt-1 h-3 w-3 rounded-full ${getHealthColor(account.fail_count)}`} />
                      <div className="min-w-0 space-y-1">
                        <div className="flex min-w-0 items-center gap-2">
                          <span className="truncate font-medium">{account.name}</span>
                          {!account.active && <Badge variant="secondary">Paused</Badge>}
                        </div>
                        {(account.account_label || account.external_account_id) && (
                          <p className="truncate text-xs text-slate-500">
                            {account.account_label || account.external_account_id}
                          </p>
                        )}
                      </div>
                    </div>
                  </TableCell>
                  <TableCell className="align-top">
                    <div className="min-w-0 space-y-1">
                      <Badge variant="outline" className={`max-w-full border ${getSessionTone(account.session_status)}`}>
                        {formatSessionStatus(account.session_status)}
                      </Badge>
                      {shouldShowRoutingBadge(account.session_status, account.routing_state) && (
                        <Badge variant="outline" className={`max-w-full border ${getRoutingTone(account.routing_state)}`}>
                          {formatRoutingState(account.routing_state)}
                        </Badge>
                      )}
                      {account.cooldown_until && account.routing_state === 'cooling_down' && (
                        <p className="truncate text-xs text-slate-500">
                          Cooldown until {formatUsageReset(account.cooldown_until)}
                        </p>
                      )}
                      {account.session_error && (
                        <p className="truncate text-xs text-slate-500">
                          {account.session_error}
                        </p>
                      )}
                      {account.last_routing_error && account.last_routing_error !== account.session_error && (
                        <p className="truncate text-xs text-slate-500">
                          {account.last_routing_error}
                        </p>
                      )}
                    </div>
                  </TableCell>
                  <TableCell className="align-top text-slate-400">
                    <span className="block truncate">{getProxyLabel(account)}</span>
                  </TableCell>
                  <TableCell className="align-top">{account.request_count}</TableCell>
                  <TableCell className="align-top text-green-500">{account.success_count}</TableCell>
                  <TableCell className="align-top text-red-500">{account.fail_count}</TableCell>
                  {activeProviderTab === PROVIDER_CODEX && (
                    <TableCell className="align-top">
                      {renderCodexUsage(account)}
                    </TableCell>
                  )}
                  <TableCell className="align-top text-slate-400">
                    {account.last_used?.slice(0, 10) || '-'}
                  </TableCell>
                  <TableCell className="align-top">
                    <div className="flex gap-2">
                      <Button variant="ghost" size="sm" onClick={() => openEditDialog(account)}>
                        <Edit className="h-4 w-4" />
                      </Button>
                      <Button variant="ghost" size="sm" onClick={() => setDeleteTarget(account)}>
                        <Trash2 className="h-4 w-4 text-red-500" />
                      </Button>
                    </div>
                  </TableCell>
                </TableRow>
              )) : (
                <TableRow>
                  <TableCell colSpan={activeProviderTab === PROVIDER_CODEX ? 9 : 8} className="py-8 text-center text-sm text-slate-500">
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

      <Dialog
        open={dialogOpen}
        onOpenChange={(open) => {
          setDialogOpen(open);
          if (!open) setError('');
        }}
      >
        <DialogContent className="max-h-[90vh] w-[calc(100vw-2rem)] overflow-x-hidden overflow-y-auto sm:w-[min(96vw,72rem)] sm:max-w-[72rem]">
          <DialogHeader>
            <DialogTitle>{editAccount ? 'Edit Account' : 'Add Account'}</DialogTitle>
            <DialogDescription>
              {editAccount
                ? 'Manage provider account state, proxy assignment, and provider-specific actions.'
                : 'Create a Grok cookie account or a Codex account.'}
            </DialogDescription>
          </DialogHeader>

          {error && (
            <div className="rounded border border-red-500/20 bg-red-500/10 p-3 text-sm text-red-400">
              {error}
            </div>
          )}

          <div className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">Name</label>
              <Input
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder={providerSlug === PROVIDER_CODEX && !editAccount ? 'Optional for Codex login import' : 'account-1'}
                maxLength={100}
                disabled={Boolean(editAccount)}
              />
              <p className="text-xs text-slate-500">
                {providerSlug === PROVIDER_CODEX && !editAccount
                  ? 'Optional for Codex login import. If left blank, the system auto-generates a unique name from account metadata.'
                  : 'Letters, numbers, underscores, hyphens only (max 100 chars).'}
              </p>
            </div>

            <div className="space-y-2">
              <label className="text-sm font-medium">Provider</label>
              <Select
                value={providerSlug}
                onValueChange={(value) => setProviderSlug(value || PROVIDER_GROK)}
                disabled={Boolean(editAccount)}
              >
                <SelectTrigger className="w-full">
                  <SelectValue placeholder="Select provider" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value={PROVIDER_GROK}>Grok</SelectItem>
                  <SelectItem value={PROVIDER_CODEX}>Codex</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-2">
              <label className="text-sm font-medium">Active</label>
              <div className="flex items-center gap-2">
                <Button variant={active ? 'default' : 'outline'} size="sm" onClick={() => setActive(true)}>
                  Active
                </Button>
                <Button variant={!active ? 'default' : 'outline'} size="sm" onClick={() => setActive(false)}>
                  Paused
                </Button>
              </div>
            </div>

            <div className="space-y-2">
              <label className="text-sm font-medium">Proxy</label>
              <Select value={selectedProxyId} onValueChange={(value) => setSelectedProxyId(value ?? 'none')}>
                <SelectTrigger className="w-full">
                  <SelectValue placeholder="Select proxy" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="none">Direct connection</SelectItem>
                  {proxies.map((proxy) => (
                    <SelectItem key={proxy.id} value={String(proxy.id)}>
                      {proxy.url}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <p className="text-xs text-muted-foreground">
                Pin a proxy here when this account should always use one fixed exit IP. Grok and Codex both use the selected proxy when present.
              </p>
            </div>

            {providerSlug === PROVIDER_CODEX && !editAccount && (
              <div className="space-y-5 rounded-2xl border border-border/70 bg-muted/20 p-5">
                <Card size="sm" className="border border-border/70 bg-background/80 py-0 shadow-sm">
                  <CardHeader className="border-b">
                    <CardTitle>Codex Import</CardTitle>
                    <CardDescription>
                      No pre-created account needed. Start the Codex login flow, finish auth, then this dialog will create the account automatically.
                    </CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-4 py-4">
                    <div className="flex flex-wrap gap-2">
                      <Button type="button" variant="outline" onClick={handleStartCodexLogin} disabled={sessionActionPending !== null}>
                        {sessionActionPending === 'codex-login' && <RefreshCw className="mr-2 h-4 w-4 animate-spin" />}
                        Start Codex Login
                      </Button>
                      {codexLoginSession && (
                        <Button type="button" variant="outline" onClick={handleRefreshCodexLoginStatus} disabled={sessionActionPending !== null}>
                          {sessionActionPending === 'codex-status' && <RefreshCw className="mr-2 h-4 w-4 animate-spin" />}
                          Check Login Status
                        </Button>
                      )}
                    </div>
                    <p className="text-xs text-muted-foreground">
                      The selected proxy is applied to both the Codex CLI login process and the launched browser session when available.
                    </p>
                  </CardContent>
                </Card>
              </div>
            )}

            {editAccount && (
              <div className="space-y-5 rounded-2xl border border-border/70 bg-muted/20 p-5">
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div className="flex flex-wrap items-center gap-2">
                    <Badge variant="outline" className={`border ${getSessionTone(editAccount.session_status)}`}>
                      {formatSessionStatus(editAccount.session_status)}
                    </Badge>
                    {shouldShowRoutingBadge(editAccount.session_status, editAccount.routing_state) && (
                      <Badge variant="outline" className={`border ${getRoutingTone(editAccount.routing_state)}`}>
                        {formatRoutingState(editAccount.routing_state)}
                      </Badge>
                    )}
                  </div>
                  {typeof credentialPreview.expires_at === 'string' && (
                    <div className="min-w-44 text-left text-xs sm:text-right">
                      <p className="text-muted-foreground">Expires</p>
                      <p className="font-medium text-foreground">
                        {formatUsageReset(credentialPreview.expires_at)}
                      </p>
                    </div>
                  )}
                </div>

                {editAccount.provider_slug === PROVIDER_GROK ? (
                  <>
                    <Card size="sm" className="border border-border/70 bg-background/80 py-0 shadow-sm">
                      <CardHeader className="border-b">
                        <CardTitle>Browser Profile</CardTitle>
                        <CardDescription>
                          Grok login stays attached to this profile directory and can be synced back into stored cookies.
                        </CardDescription>
                      </CardHeader>
                      <CardContent className="space-y-4 py-4">
                        <div className="rounded-lg border border-border/60 bg-muted/30 p-3">
                          <p className="mb-1 text-[11px] font-medium uppercase tracking-[0.12em] text-muted-foreground">Profile Path</p>
                          <p className="break-all font-mono text-xs text-foreground">
                            {editAccount.profile_dir || `data/browser-profiles/${editAccount.name}`}
                          </p>
                        </div>
                        <div className="grid gap-2 text-xs text-muted-foreground">
                          <p>
                            Cooldown until:{' '}
                            <span className="font-medium text-foreground">
                              {editAccount.cooldown_until
                                ? formatUsageReset(editAccount.cooldown_until)
                                : '-'}
                            </span>
                          </p>
                          <p>
                            Routing streaks:{' '}
                            <span className="font-medium text-foreground">
                              RL {editAccount.rate_limit_streak} / Auth {editAccount.auth_failure_streak} / Refresh {editAccount.refresh_failure_streak}
                            </span>
                          </p>
                          {editAccount.last_routing_error && (
                            <div className="rounded-md border border-amber-500/20 bg-amber-500/10 px-3 py-2 text-amber-700 dark:text-amber-300">
                              {editAccount.last_routing_error}
                            </div>
                          )}
                        </div>
                        <div className="flex flex-wrap gap-2">
                          <Button type="button" variant="outline" onClick={handleOpenLogin} disabled={sessionActionPending !== null}>
                            {sessionActionPending === 'launch' && <RefreshCw className="mr-2 h-4 w-4 animate-spin" />}
                            Open Login Browser
                          </Button>
                          <Button type="button" variant="outline" onClick={handleSyncProfile} disabled={sessionActionPending !== null}>
                            {sessionActionPending === 'sync' && <RefreshCw className="mr-2 h-4 w-4 animate-spin" />}
                            Sync From Profile
                          </Button>
                        </div>
                      </CardContent>
                    </Card>
                    <p className="text-xs text-muted-foreground">
                      Leave cookies blank to keep the current stored cookies. Use browser sync when you want a fresh login imported safely.
                    </p>
                  </>
                ) : (
                  <>
                    <div className="grid gap-4 xl:grid-cols-2">
                      <Card size="sm" className="border border-border/70 bg-background/80 py-0 shadow-sm">
                        <CardHeader className="border-b">
                          <CardTitle>Account Snapshot</CardTitle>
                          <CardDescription>
                            Core Codex identity and token state for this account.
                          </CardDescription>
                        </CardHeader>
                        <CardContent className="grid gap-3 py-4 sm:grid-cols-2">
                          <div className="rounded-lg border border-border/60 bg-muted/30 p-3">
                            <p className="mb-1 text-[11px] font-medium uppercase tracking-[0.12em] text-muted-foreground">Email</p>
                            <p className="break-all text-sm font-medium text-foreground">
                              {typeof credentialPreview.email === 'string' ? credentialPreview.email : '-'}
                            </p>
                          </div>
                          <div className="rounded-lg border border-border/60 bg-muted/30 p-3">
                            <p className="mb-1 text-[11px] font-medium uppercase tracking-[0.12em] text-muted-foreground">Refresh Token</p>
                            <p className="text-sm font-medium text-foreground">
                              {credentialPreview.has_refresh_token ? 'Available' : 'Missing'}
                            </p>
                          </div>
                          <div className="rounded-lg border border-border/60 bg-muted/30 p-3 sm:col-span-2">
                            <p className="mb-1 text-[11px] font-medium uppercase tracking-[0.12em] text-muted-foreground">External Account</p>
                            <p className="break-all font-mono text-xs text-foreground">
                              {typeof credentialPreview.account_id === 'string' ? credentialPreview.account_id : '-'}
                            </p>
                          </div>
                        </CardContent>
                      </Card>

                      <Card size="sm" className="border border-border/70 bg-background/80 py-0 shadow-sm">
                        <CardHeader className="border-b">
                          <CardTitle>Routing Health</CardTitle>
                          <CardDescription>
                            Cooldown, failure streaks, and the last routing signal for this account.
                          </CardDescription>
                        </CardHeader>
                        <CardContent className="space-y-3 py-4">
                          <div className="grid gap-3 sm:grid-cols-2">
                            <div className="rounded-lg border border-border/60 bg-muted/30 p-3">
                              <p className="mb-1 text-[11px] font-medium uppercase tracking-[0.12em] text-muted-foreground">Cooldown</p>
                              <p className="text-sm font-medium text-foreground">
                                {editAccount.cooldown_until
                                  ? formatUsageReset(editAccount.cooldown_until)
                                  : '-'}
                              </p>
                            </div>
                            <div className="rounded-lg border border-border/60 bg-muted/30 p-3">
                              <p className="mb-1 text-[11px] font-medium uppercase tracking-[0.12em] text-muted-foreground">Routing Streaks</p>
                              <p className="text-sm font-medium text-foreground">
                                RL {editAccount.rate_limit_streak} / Auth {editAccount.auth_failure_streak} / Refresh {editAccount.refresh_failure_streak}
                              </p>
                            </div>
                          </div>
                          {editAccount.last_routing_error ? (
                            <div className="rounded-lg border border-amber-500/20 bg-amber-500/10 p-3 text-sm text-amber-700 dark:text-amber-300">
                              <p className="mb-1 text-[11px] font-medium uppercase tracking-[0.12em]">Last Routing Error</p>
                              <p className="break-words">{editAccount.last_routing_error}</p>
                            </div>
                          ) : (
                            <div className="rounded-lg border border-emerald-500/20 bg-emerald-500/10 p-3 text-sm text-emerald-700 dark:text-emerald-300">
                              No active routing error.
                            </div>
                          )}
                        </CardContent>
                      </Card>
                    </div>

                    <Card size="sm" className="border border-border/70 bg-background/80 py-0 shadow-sm">
                      <CardHeader className="border-b">
                        <CardTitle>Account Actions</CardTitle>
                        <CardDescription>
                          Login and token maintenance actions for this Codex account.
                        </CardDescription>
                      </CardHeader>
                      <CardContent className="py-4">
                        <div className="flex flex-wrap gap-2">
                          <Button type="button" variant="outline" onClick={handleStartCodexLogin} disabled={sessionActionPending !== null}>
                            {sessionActionPending === 'codex-login' && <RefreshCw className="mr-2 h-4 w-4 animate-spin" />}
                            Start Codex Login
                          </Button>
                          <Button type="button" variant="outline" onClick={handleRefreshCodexLoginStatus} disabled={sessionActionPending !== null}>
                            {sessionActionPending === 'codex-status' && <RefreshCw className="mr-2 h-4 w-4 animate-spin" />}
                            Check Login Status
                          </Button>
                          <Button type="button" variant="outline" onClick={handleRefreshCodexToken} disabled={sessionActionPending !== null}>
                            {sessionActionPending === 'codex-refresh' && <RefreshCw className="mr-2 h-4 w-4 animate-spin" />}
                            Refresh Token
                          </Button>
                        </div>
                      </CardContent>
                    </Card>

                    <Card size="sm" className="border border-border/70 bg-background/80 py-0 shadow-sm">
                      <CardHeader className="border-b">
                        <CardTitle>Usage Snapshot</CardTitle>
                        <CardDescription>
                          Live Codex quota usage pulled from upstream for this account.
                        </CardDescription>
                      </CardHeader>
                      <CardContent className="py-4">
                        {renderCodexUsage(editAccount, 'detail')}
                      </CardContent>
                    </Card>

                  </>
                )}
              </div>
            )}

            {providerSlug === PROVIDER_CODEX && codexLoginSession && (
              <Card size="sm" className="border border-border/70 bg-background/80 py-0 shadow-sm">
                <CardHeader className="border-b">
                  <CardTitle>Codex Login Session</CardTitle>
                  <CardDescription>
                    Live state for the current device login flow.
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-3 py-4 text-sm">
                  <div className="flex flex-wrap items-center gap-2">
                    <Badge variant="outline" className="border-border text-foreground">
                      {formatCodexLoginStatus(codexLoginSession.status)}
                    </Badge>
                    {codexLoginPending && <span className="text-xs text-muted-foreground">Polling automatically every 2s.</span>}
                  </div>

                  <div className="rounded-lg border border-border/60 bg-muted/25 p-3">
                    <p className="mb-1 text-[11px] font-medium uppercase tracking-[0.12em] text-muted-foreground">Verification URL</p>
                    {codexLoginSession.verification_url ? (
                      <a
                        href={codexLoginSession.verification_url}
                        target="_blank"
                        rel="noreferrer"
                        className="break-all text-sm font-medium text-primary underline underline-offset-2"
                      >
                        {codexLoginSession.verification_url}
                      </a>
                    ) : (
                      <p className="text-sm text-muted-foreground">Waiting for login link...</p>
                    )}
                  </div>

                  <div className="grid gap-3 sm:grid-cols-2">
                    <div className="rounded-lg border border-border/60 bg-muted/25 p-3">
                      <p className="mb-1 text-[11px] font-medium uppercase tracking-[0.12em] text-muted-foreground">Code</p>
                      <p className="break-all font-mono text-sm text-foreground">
                        {codexLoginSession.user_code || 'Waiting for code...'}
                      </p>
                    </div>
                    <div className="rounded-lg border border-border/60 bg-muted/25 p-3">
                      <p className="mb-1 text-[11px] font-medium uppercase tracking-[0.12em] text-muted-foreground">Expires</p>
                      <p className="text-sm text-foreground">
                        {codexLoginSession.expires_at
                          ? formatUsageReset(codexLoginSession.expires_at)
                          : '-'}
                      </p>
                    </div>
                  </div>

                  {codexLoginSession.message && (
                    <div className="rounded-lg border border-border/60 bg-muted/25 p-3 text-sm text-foreground">
                      {codexLoginSession.message}
                    </div>
                  )}

                  {codexLoginPending && (
                    <div className="min-w-0 space-y-2 rounded-lg border border-border/60 bg-muted/25 p-4">
                      <p className="text-sm text-foreground">
                        SSH / remote case: paste the full <code>http://localhost:1455/auth/callback?...</code> URL here.
                      </p>
                      <Input
                        value={codexManualCallbackUrl}
                        onChange={(e) => setCodexManualCallbackUrl(e.target.value)}
                        placeholder="http://localhost:1455/auth/callback?code=...&scope=...&state=..."
                        className="w-full min-w-0 max-w-full font-mono text-[11px]"
                      />
                      <div className="flex flex-wrap gap-2">
                        <Button
                          type="button"
                          variant="outline"
                          onClick={handleSubmitCodexCallback}
                          disabled={sessionActionPending !== null}
                        >
                          {sessionActionPending === 'codex-callback' && <RefreshCw className="mr-2 h-4 w-4 animate-spin" />}
                          Submit Callback URL
                        </Button>
                        <Button
                          type="button"
                          variant="outline"
                          onClick={handleRefreshCodexLoginStatus}
                          disabled={sessionActionPending !== null}
                        >
                          {sessionActionPending === 'codex-status' && <RefreshCw className="mr-2 h-4 w-4 animate-spin" />}
                          Refresh Status
                        </Button>
                      </div>
                    </div>
                  )}

                  <div className="rounded-lg border border-border/60 bg-muted/25 p-3">
                    <p className="mb-1 text-[11px] font-medium uppercase tracking-[0.12em] text-muted-foreground">Local Command</p>
                    <p className="break-all font-mono text-[11px] text-foreground">{codexLoginSession.command}</p>
                  </div>
                </CardContent>
              </Card>
            )}

            {providerSlug === PROVIDER_GROK ? (
              <div className="space-y-2">
                <label className="text-sm font-medium">Grok Cookies</label>
                <Textarea
                  value={credentials}
                  onChange={(e) => setCredentials(e.target.value)}
                  placeholder='Optional: raw cookie string or {"sso":"...","sso-rw":"..."}'
                  className="h-44 resize-y font-mono [field-sizing:fixed]"
                />
                <p className="text-xs text-slate-500">
                  On edit, leave blank to keep the existing credential bundle.
                </p>
              </div>
            ) : (
              <div className="space-y-2">
                <label className="text-sm font-medium">Advanced Token Import</label>
                <Textarea
                  value={credentials}
                  onChange={(e) => setCredentials(e.target.value)}
                  placeholder='Optional advanced import: {"access_token":"...","refresh_token":"..."}'
                  className="h-36 resize-y font-mono [field-sizing:fixed]"
                />
                <p className="text-xs text-muted-foreground">
                  Preferred flow: Start Codex Login above, finish auth, then the account is created automatically. Raw token JSON stays available for advanced recovery/import cases.
                </p>
              </div>
            )}
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => setDialogOpen(false)}>
              Cancel
            </Button>
            <Button onClick={editAccount ? handleUpdate : handleCreate}>
              {editAccount ? 'Save' : providerSlug === PROVIDER_CODEX ? 'Import Tokens' : 'Create'}
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
