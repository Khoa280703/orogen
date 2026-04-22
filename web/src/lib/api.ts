import { buildApiUrl } from '@/lib/api-base-url';

const ADMIN_TOKEN_KEY = 'admin_token';
const CSRF_TOKEN_KEY = 'csrfToken';
const ADMIN_TOKEN_EVENT = 'admin-token-change';

export interface ProxySummary {
  id: number;
  url: string;
  label: string | null;
  active: boolean;
  created_at: string | null;
  assigned_accounts: number;
}

export interface ProviderSummary {
  id: number;
  name: string;
  slug: string;
  active: boolean;
  created_at: string;
}

export interface ProviderModelSummary {
  id: number;
  provider_id: number;
  provider_name: string;
  name: string;
  slug: string;
  description: string | null;
  active: boolean;
  sort_order: number;
  created_at: string;
}

export interface AccountSummary {
  id: number;
  name: string;
  provider_slug: string;
  credential_preview: Record<string, unknown>;
  account_label: string | null;
  external_account_id: string | null;
  auth_mode: string | null;
  active: boolean;
  proxy_id: number | null;
  profile_dir: string | null;
  session_status: string;
  session_error: string | null;
  request_count: number;
  fail_count: number;
  success_count: number;
  last_used: string | null;
  created_at: string | null;
  session_checked_at: string | null;
  cookies_synced_at: string | null;
  routing_state: string;
  cooldown_until: string | null;
  last_routing_error: string | null;
  rate_limit_streak: number;
  auth_failure_streak: number;
  refresh_failure_streak: number;
}

export interface AccountUsageQuota {
  used: number;
  total: number;
  remaining: number;
  remaining_percentage: number;
  reset_at: string | null;
  unlimited: boolean;
}

export interface AccountUsageSummary {
  account_id: number;
  provider_slug: string;
  supported: boolean;
  fetched_at: string;
  plan: string | null;
  limit_reached: boolean | null;
  message: string | null;
  quotas: Record<string, AccountUsageQuota>;
}

export interface CodexLoginSession {
  session_id: string;
  account_id: number;
  status: string;
  verification_url: string;
  user_code: string | null;
  expires_at: string | null;
  command: string;
  message: string | null;
}

export interface PublicGatewayModel {
  id: string;
  object?: string;
  display_name?: string;
  description?: string | null;
  owned_by?: string;
}

export interface UsageLogEntry {
  id: number | null;
  api_key_id: number | null;
  plan_id: number | null;
  account_id: number | null;
  provider_slug: string | null;
  model: string | null;
  status: string | null;
  latency_ms: number | null;
  prompt_tokens: number | null;
  completion_tokens: number | null;
  cached_tokens: number | null;
  credits_used: number | null;
  estimated_usage: boolean | null;
  created_at: string | null;
}

export interface UsageLogFilters {
  statuses: string[];
  models: string[];
  providers: string[];
}

export interface UsageLogAggregates {
  prompt_tokens: number;
  completion_tokens: number;
  cached_tokens: number;
  credits_used: number;
}

export interface UsageLogBreakdownRow {
  label: string;
  requests: number;
  prompt_tokens: number;
  completion_tokens: number;
  cached_tokens: number;
  credits_used: number;
}

export interface AdminListResponse<T> {
  items: T[];
  total: number;
  page?: number;
  limit?: number;
}

export interface AdminConversationListItem {
  id: number;
  user_id: number;
  user_email: string;
  user_name: string | null;
  title: string | null;
  model_slug: string | null;
  active: boolean;
  created_at: string | null;
  updated_at: string | null;
  message_count: number;
}

export interface AdminConversationDetail {
  id: number;
  user_id: number;
  user_email: string;
  user_name: string | null;
  title: string | null;
  model_slug: string | null;
  active: boolean;
  created_at: string | null;
  updated_at: string | null;
  messages: Array<{
    id: number;
    role: string;
    content: string;
    created_at: string | null;
  }>;
}

export interface AdminImageListItem {
  id: number;
  user_id: number;
  user_email: string;
  user_name: string | null;
  prompt: string;
  model_slug: string;
  status: string;
  image_count: number;
  error_message: string | null;
  created_at: string | null;
}

export interface AdminImageDetail {
  id: number;
  user_id: number;
  user_email: string;
  user_name: string | null;
  prompt: string;
  model_slug: string;
  status: string;
  result_urls: Array<{ id?: string; url?: string }>;
  error_message: string | null;
  created_at: string | null;
}

export type AccountCookiesInput = string | Record<string, unknown>;

export function getAdminToken(): string | null {
  if (typeof window === 'undefined') return null;
  return localStorage.getItem(ADMIN_TOKEN_KEY);
}

export function setAdminToken(token: string): void {
  if (typeof window === 'undefined') return;
  localStorage.setItem(ADMIN_TOKEN_KEY, token);
  window.dispatchEvent(new Event(ADMIN_TOKEN_EVENT));
}

export function clearAdminToken(): void {
  if (typeof window === 'undefined') return;
  localStorage.removeItem(ADMIN_TOKEN_KEY);
  localStorage.removeItem(CSRF_TOKEN_KEY);
  window.dispatchEvent(new Event(ADMIN_TOKEN_EVENT));
}

export function getCsrfToken(): string | null {
  if (typeof window === 'undefined') return null;
  return localStorage.getItem(CSRF_TOKEN_KEY);
}

export function setCsrfToken(token: string): void {
  if (typeof window === 'undefined') return;
  localStorage.setItem(CSRF_TOKEN_KEY, token);
}

function isHtmlPayload(value: string): boolean {
  const trimmed = value.trim().toLowerCase();
  return trimmed.startsWith('<!doctype html') || trimmed.startsWith('<html');
}

function formatTextError(status: number, text: string): string {
  const trimmed = text.trim();
  if (!trimmed) {
    return '';
  }

  if (isHtmlPayload(trimmed)) {
    return 'API returned HTML instead of JSON';
  }

  const singleLine = trimmed.replace(/\s+/g, ' ');
  return singleLine.length > 240 ? `${singleLine.slice(0, 237)}...` : singleLine;
}

async function readJsonOrThrow<T>(response: Response): Promise<T> {
  if (response.status === 204 || response.headers.get('content-length') === '0') {
    return undefined as T;
  }

  const contentType = response.headers.get('content-type') || '';
  const text = await response.text();

  if (!text.trim()) {
    return undefined as T;
  }

  if (contentType.includes('application/json')) {
    return JSON.parse(text) as T;
  }

  if (isHtmlPayload(text)) {
    throw new Error('API returned HTML instead of JSON');
  }

  return text as T;
}

async function publicRequest<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
  const response = await fetch(buildApiUrl(endpoint), {
    ...options,
    credentials: 'include',
  });

  if (!response.ok) {
    const contentType = response.headers.get('content-type') || '';
    let message = `API error: ${response.status}`;
    try {
      if (contentType.includes('application/json')) {
        const text = await response.text();
        if (text.trim()) {
          const data = JSON.parse(text);
          const detail =
            (typeof data?.error === 'string' && data.error) ||
            (typeof data?.error?.message === 'string' && data.error.message) ||
            (typeof data?.message === 'string' && data.message) ||
            null;
          if (detail) {
            message = `${message} - ${detail}`;
          }
        }
      } else {
        const text = await response.text();
        const detail = formatTextError(response.status, text);
        if (detail) {
          message = `${message} - ${detail}`;
        }
      }
    } catch {
      // Keep default message if parsing fails.
    }
    throw new Error(message);
  }

  return readJsonOrThrow<T>(response);
}

export async function refreshCsrfToken(): Promise<boolean> {
  const token = getAdminToken();
  if (!token) return false;

  try {
    const response = await fetch(buildApiUrl('/admin/csrf-token'), {
      credentials: 'include',
      headers: {
        Authorization: `Bearer ${token}`,
      },
    });
    if (response.status === 401) {
      return false;
    }
    if (response.ok) {
      const data = await response.json();
      setCsrfToken(data.token);
      return true;
    }
  } catch (error) {
    void error;
  }
  return false;
}

export async function apiRequest<T>(
  endpoint: string,
  options: RequestInit = {}
): Promise<T> {
  const token = getAdminToken();

  if (!token) {
    throw new Error('Missing admin token');
  }

  // Refresh CSRF token before state-changing requests
  const isStateChanging = options.method && !['GET', 'HEAD', 'OPTIONS'].includes(options.method);
  if (isStateChanging) {
    const csrfRefreshed = await refreshCsrfToken();
    if (!csrfRefreshed) {
      // Token might be invalid, but don't redirect yet - let the actual request fail
      console.warn('Failed to refresh CSRF token, will try anyway');
    }
  }

  const csrfToken = getCsrfToken();

  const headers: HeadersInit = {
    'Content-Type': 'application/json',
    ...(token ? { Authorization: `Bearer ${token}` } : {}),
    ...(csrfToken ? { 'X-CSRF-Token': csrfToken } : {}),
    ...options.headers,
  };

  const url = buildApiUrl(endpoint);

  const response = await fetch(url, {
    ...options,
    credentials: 'include',
    headers,
  });

  const readErrorMessage = async (res: Response): Promise<string | null> => {
    try {
      const contentType = res.headers.get('content-type') || '';
      if (contentType.includes('application/json')) {
        const text = await res.text();
        if (!text.trim()) return null;
        const data = JSON.parse(text);
        if (typeof data === 'string') return data;
        if (data?.error && typeof data.error === 'string') return data.error;
        if (data?.message && typeof data.message === 'string') return data.message;
        return JSON.stringify(data);
      }

      const text = await res.text();
      return formatTextError(res.status, text);
    } catch {
      return null;
    }
  };

  if (!response.ok) {
    if (response.status === 401) {
      // Only redirect if we have a token (not if it was already missing)
      if (token) {
        clearAdminToken();
        if (typeof window !== 'undefined') {
          window.location.href = '/login';
        }
      }
      throw new Error('Unauthorized');
    }
    if (response.status === 403) {
      // CSRF token invalid, refresh and retry once
      if (isStateChanging) {
        await refreshCsrfToken();
        const newCsrfToken = getCsrfToken();
        const retryResponse = await fetch(url, {
          ...options,
          credentials: 'include',
          headers: {
            'Content-Type': 'application/json',
            ...(token ? { Authorization: `Bearer ${token}` } : {}),
            ...(newCsrfToken ? { 'X-CSRF-Token': newCsrfToken } : {}),
            ...options.headers,
          },
        });
        if (retryResponse.ok) {
          return readJsonOrThrow<T>(retryResponse);
        }
      }
      throw new Error('CSRF token invalid');
    }
    const errorMessage = await readErrorMessage(response);
    throw new Error(errorMessage ? `API error: ${response.status} - ${errorMessage}` : `API error: ${response.status}`);
  }

  return readJsonOrThrow<T>(response);
}

// Proxies
export async function listProxies() {
  return apiRequest<ProxySummary[]>('/admin/proxies');
}

export async function createProxy(url: string, label?: string) {
  return apiRequest<{ id: number }>('/admin/proxies', {
    method: 'POST',
    body: JSON.stringify({ url, label }),
  });
}

export async function updateProxy(id: number, data: { url?: string; label?: string; active?: boolean }) {
  return apiRequest<{ success: boolean }>(`/admin/proxies/${id}`, {
    method: 'PUT',
    body: JSON.stringify(data),
  });
}

export async function deleteProxy(id: number) {
  return apiRequest<{ success: boolean }>(`/admin/proxies/${id}`, {
    method: 'DELETE',
  });
}

// Accounts
export async function listAccounts() {
  return apiRequest<AccountSummary[]>('/admin/accounts');
}

export async function createAccount(data: {
  name: string;
  providerSlug?: string;
  credentials?: AccountCookiesInput;
  proxyId?: number | null;
}) {
  return apiRequest<{ id: number }>('/admin/accounts', {
    method: 'POST',
    body: JSON.stringify({
      name: data.name,
      provider_slug: data.providerSlug,
      credentials: data.credentials,
      proxy_id: data.proxyId,
    }),
  });
}

export async function updateAccount(id: number, data: {
  credentials?: AccountCookiesInput;
  active?: boolean;
  proxyId?: number | null;
}) {
  return apiRequest<{ success: boolean }>(`/admin/accounts/${id}`, {
    method: 'PUT',
    body: JSON.stringify({
      credentials: data.credentials,
      active: data.active,
      proxy_id: data.proxyId,
    }),
  });
}

export async function deleteAccount(id: number) {
  return apiRequest<{ success: boolean }>(`/admin/accounts/${id}`, {
    method: 'DELETE',
  });
}

export async function getAccountUsage(id: number) {
  return apiRequest<AccountUsageSummary>(`/admin/accounts/${id}/usage`);
}

export async function openAccountLoginBrowser(id: number) {
  return apiRequest<{ success: boolean; profile_dir: string; pid?: number; message?: string }>(
    `/admin/accounts/${id}/open-login-browser`,
    {
      method: 'POST',
    }
  );
}

export async function syncAccountProfile(id: number) {
  return apiRequest<{ success: boolean; profile_dir: string; message?: string }>(
    `/admin/accounts/${id}/sync-profile`,
    {
      method: 'POST',
    }
  );
}

export async function startCodexAccountLogin(id: number) {
  return apiRequest<{ success: boolean; session: CodexLoginSession }>(
    `/admin/accounts/${id}/start-codex-login`,
    {
      method: 'POST',
    }
  );
}

export async function startCodexImportLogin(data: { name?: string; proxyId?: number | null }) {
  return apiRequest<{ success: boolean; session: CodexLoginSession }>(
    '/admin/accounts/codex-import/start',
    {
      method: 'POST',
      body: JSON.stringify({
        name: data.name?.trim() || null,
        proxy_id: data.proxyId ?? null,
      }),
    }
  );
}

export async function getCodexAccountLoginStatus(id: number) {
  return apiRequest<{ success: boolean; session: CodexLoginSession }>(
    `/admin/accounts/${id}/codex-login-status`
  );
}

export async function getCodexImportLoginStatus(sessionId: string) {
  return apiRequest<{ success: boolean; session: CodexLoginSession }>(
    `/admin/accounts/codex-import/${sessionId}`
  );
}

export async function submitCodexAccountCallback(id: number, callbackUrl: string) {
  return apiRequest<{ success: boolean; session: CodexLoginSession }>(
    `/admin/accounts/${id}/complete-codex-login`,
    {
      method: 'POST',
      body: JSON.stringify({ callback_url: callbackUrl }),
    }
  );
}

export async function submitCodexImportCallback(sessionId: string, callbackUrl: string) {
  return apiRequest<{ success: boolean; session: CodexLoginSession }>(
    `/admin/accounts/codex-import/${sessionId}/complete`,
    {
      method: 'POST',
      body: JSON.stringify({ callback_url: callbackUrl }),
    }
  );
}

export async function refreshCodexAccountToken(id: number) {
  return apiRequest<{ success: boolean; credential_preview: Record<string, unknown>; message?: string }>(
    `/admin/accounts/${id}/refresh-codex-token`,
    {
      method: 'POST',
    }
  );
}

export async function listPublicGatewayModels(apiKey?: string) {
  const headers: HeadersInit = {};
  if (apiKey?.trim()) {
    headers.Authorization = `Bearer ${apiKey.trim()}`;
  }

  return publicRequest<{ data?: PublicGatewayModel[] }>('/v1/models', {
    headers,
  });
}

// Providers
export async function listProviders() {
  return apiRequest<{ data: ProviderSummary[] }>('/admin/providers');
}

export async function createProvider(data: { name: string; slug: string }) {
  return apiRequest<{ data: ProviderSummary }>('/admin/providers', {
    method: 'POST',
    body: JSON.stringify(data),
  });
}

export async function updateProvider(id: number, data: { name?: string; active?: boolean }) {
  return apiRequest<{ data: ProviderSummary }>(`/admin/providers/${id}`, {
    method: 'PUT',
    body: JSON.stringify(data),
  });
}

// Models
export async function listProviderModels() {
  return apiRequest<{ data: ProviderModelSummary[] }>('/admin/models');
}

export async function createProviderModel(data: {
  provider_id: number;
  name: string;
  slug: string;
  description?: string;
  sort_order?: number;
}) {
  return apiRequest<{ data: ProviderModelSummary }>('/admin/models', {
    method: 'POST',
    body: JSON.stringify(data),
  });
}

export async function updateProviderModel(
  id: number,
  data: { name?: string; description?: string; active?: boolean; sort_order?: number }
) {
  return apiRequest<{ data: ProviderModelSummary }>(`/admin/models/${id}`, {
    method: 'PUT',
    body: JSON.stringify(data),
  });
}

export async function deactivateProviderModel(id: number) {
  return apiRequest<{ success: boolean; message?: string }>(`/admin/models/${id}`, {
    method: 'DELETE',
  });
}

// API Keys
export async function listApiKeys() {
  return apiRequest<[]>('/admin/api-keys');
}

export async function createApiKey(data: {
  label?: string;
  quotaPerDay?: number;
  dailyCreditLimit?: number;
  monthlyCreditLimit?: number;
  maxInputTokens?: number;
  maxOutputTokens?: number;
  planId?: number;
}) {
  return apiRequest<{ id: number; key: string }>('/admin/api-keys', {
    method: 'POST',
    body: JSON.stringify({
      label: data.label,
      quota_per_day: data.quotaPerDay,
      daily_credit_limit: data.dailyCreditLimit,
      monthly_credit_limit: data.monthlyCreditLimit,
      max_input_tokens: data.maxInputTokens,
      max_output_tokens: data.maxOutputTokens,
      plan_id: data.planId,
    }),
  });
}

export async function updateApiKey(id: number, data: {
  label?: string;
  active?: boolean;
  quotaPerDay?: number;
  dailyCreditLimit?: number;
  monthlyCreditLimit?: number;
  maxInputTokens?: number;
  maxOutputTokens?: number;
  planId?: number;
}) {
  return apiRequest<{ success: boolean }>(`/admin/api-keys/${id}`, {
    method: 'PUT',
    body: JSON.stringify({
      ...data,
      quota_per_day: data.quotaPerDay,
      daily_credit_limit: data.dailyCreditLimit,
      monthly_credit_limit: data.monthlyCreditLimit,
      max_input_tokens: data.maxInputTokens,
      max_output_tokens: data.maxOutputTokens,
      plan_id: data.planId,
    }),
  });
}

export async function deleteApiKey(id: number) {
  return apiRequest<{ success: boolean }>(`/admin/api-keys/${id}`, {
    method: 'DELETE',
  });
}

export async function deletePlan(id: number) {
  return apiRequest<{ success: boolean }>(`/admin/plans/${id}`, {
    method: 'DELETE',
  });
}

// Stats
export async function getStatsOverview() {
  return apiRequest<{
    total_accounts: number;
    active_accounts: number;
    requests_today: number;
    errors_today: number;
    total_conversations: number;
    total_image_generations: number;
  }>('/admin/stats/overview');
}

export async function getDailyUsage(days = 7) {
  return apiRequest<[]>(`/admin/stats/usage?days=${days}`);
}

export async function getUsageLogs(params?: {
  page?: number;
  limit?: number;
  search?: string;
  status?: string;
  model?: string;
  provider?: string;
}) {
  const query = new URLSearchParams({
    page: String(params?.page ?? 1),
    limit: String(params?.limit ?? 20),
  });
  if (params?.search?.trim()) query.set('search', params.search.trim());
  if (params?.status?.trim() && params.status !== 'all') query.set('status', params.status.trim());
  if (params?.model?.trim() && params.model !== 'all') query.set('model', params.model.trim());
  if (params?.provider?.trim() && params.provider !== 'all') query.set('provider', params.provider.trim());
  return apiRequest<{
    logs: UsageLogEntry[];
    total: number;
    page: number;
    limit: number;
    filters: UsageLogFilters;
    aggregates: UsageLogAggregates;
    breakdowns: {
      providers: UsageLogBreakdownRow[];
      models: UsageLogBreakdownRow[];
    };
  }>(`/admin/stats/logs?${query.toString()}`);
}

export async function adminFetch<T>(endpoint: string, options: RequestInit = {}) {
  return apiRequest<T>(endpoint, options);
}

export async function listAdminConversations(search = '', model = '', limit = 50, offset = 0) {
  const params = new URLSearchParams({ limit: String(limit), offset: String(offset) });
  if (search.trim()) params.set('search', search.trim());
  if (model.trim() && model !== 'all') params.set('model', model.trim());
  return adminFetch<{ items: AdminConversationListItem[]; total: number }>(`/admin/conversations?${params.toString()}`);
}

export async function getAdminConversationDetail(id: number) {
  return adminFetch<AdminConversationDetail>(`/admin/conversations/${id}`);
}

export async function deleteAdminConversation(id: number) {
  return adminFetch<{ success: boolean }>(`/admin/conversations/${id}`, { method: 'DELETE' });
}

export async function listAdminImages(search = '', status = '', limit = 50, offset = 0) {
  const params = new URLSearchParams({ limit: String(limit), offset: String(offset) });
  if (search.trim()) params.set('search', search.trim());
  if (status.trim() && status !== 'all') params.set('status', status.trim());
  return adminFetch<{ items: AdminImageListItem[]; total: number }>(`/admin/images?${params.toString()}`);
}

export async function getAdminImageDetail(id: number) {
  return adminFetch<AdminImageDetail>(`/admin/images/${id}`);
}

export async function deleteAdminImage(id: number) {
  return adminFetch<{ success: boolean }>(`/admin/images/${id}`, { method: 'DELETE' });
}

export function subscribeToAdminToken(callback: () => void) {
  if (typeof window === 'undefined') {
    return () => {};
  }

  const listener = () => callback();
  window.addEventListener('storage', listener);
  window.addEventListener(ADMIN_TOKEN_EVENT, listener);

  return () => {
    window.removeEventListener('storage', listener);
    window.removeEventListener(ADMIN_TOKEN_EVENT, listener);
  };
}
