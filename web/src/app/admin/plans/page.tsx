'use client';

import { useEffect, useMemo, useState } from 'react';
import { AdminTablePagination } from '@/components/admin/admin-table-pagination';
import { AdminTableToolbar } from '@/components/admin/admin-table-toolbar';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { ConfirmActionDialog } from '@/components/confirm-action-dialog';
import {
  Table, TableBody, TableCell, TableHead, TableHeader, TableRow,
} from '@/components/ui/table';
import { adminFetch, deletePlan } from '@/lib/api';
import { Plus, Edit, Check, X, Trash2 } from 'lucide-react';
import { PlanFormDialog } from './plan-form-dialog';
import type { PlanFormData } from './plan-form-dialog';

interface PlanFeatures {
  streaming?: boolean;
  priority?: boolean;
  dedicated_support?: boolean;
  rate_limit?: string;
  quota?: {
    daily_credits?: number;
    monthly_credits?: number;
    max_input_tokens_per_request?: number;
    max_output_tokens_per_request?: number;
    pricing?: {
      default?: {
        input_per_token?: number;
        output_per_token?: number;
        cached_input_per_token?: number;
      };
    };
  };
  model_limits?: Record<string, {
    chat_per_day?: number;
    image_per_day?: number;
    video_per_day?: number;
  }>;
  [key: string]: unknown;
}

interface Plan {
  id: number;
  name: string;
  slug: string;
  requests_per_day: number | null;
  requests_per_month: number | null;
  price_usd: string | null;
  price_vnd: number | null;
  features: PlanFeatures | null;
  active: boolean;
  sort_order: number;
}

function formatLimit(value?: number | null) {
  if (value === null || value === undefined || value === 0) return 'Unlimited';
  if (value === -1) return '\u221E';
  return value.toLocaleString();
}

function describeCredits(features: PlanFeatures | null) {
  const quota = features?.quota;
  if (!quota) return 'Not set';
  const daily = quota.daily_credits ? `D ${formatLimit(quota.daily_credits)}` : null;
  const monthly = quota.monthly_credits ? `M ${formatLimit(quota.monthly_credits)}` : null;
  return [daily, monthly].filter(Boolean).join(' · ') || 'Not set';
}

function describeTokenGuard(features: PlanFeatures | null) {
  const quota = features?.quota;
  if (!quota) return 'Not set';
  const input = quota.max_input_tokens_per_request
    ? `In ${formatLimit(quota.max_input_tokens_per_request)}`
    : null;
  const output = quota.max_output_tokens_per_request
    ? `Out ${formatLimit(quota.max_output_tokens_per_request)}`
    : null;
  return [input, output].filter(Boolean).join(' · ') || 'Not set';
}

export default function PlansPage() {
  const [plans, setPlans] = useState<Plan[]>([]);
  const [loading, setLoading] = useState(true);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [createOpen, setCreateOpen] = useState(false);
  const [editState, setEditState] = useState<{ open: boolean; planId: number | null; data: Partial<PlanFormData> }>({
    open: false, planId: null, data: {},
  });
  const [deleteTarget, setDeleteTarget] = useState<Plan | null>(null);
  const [deletePending, setDeletePending] = useState(false);
  const [search, setSearch] = useState('');
  const [statusFilter, setStatusFilter] = useState('all');
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);

  useEffect(() => { loadPlans(); }, []);
  useEffect(() => { setPage(1); }, [search, statusFilter]);

  const loadPlans = async () => {
    try {
      setErrorMessage(null);
      const data = await adminFetch<Plan[]>('/admin/plans');
      setPlans(data);
    } catch (error) {
      setErrorMessage(error instanceof Error ? error.message : 'Failed to load plans.');
    } finally { setLoading(false); }
  };

  const handleToggleActive = async (id: number, currentActive: boolean) => {
    try {
      setErrorMessage(null);
      await adminFetch(`/admin/plans/${id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ active: !currentActive }),
      });
      loadPlans();
    } catch (error) {
      setErrorMessage(error instanceof Error ? error.message : 'Failed to toggle plan.');
    }
  };

  const handleDelete = async () => {
    if (!deleteTarget) return;
    setDeletePending(true);
    try {
      await deletePlan(deleteTarget.id);
      setDeleteTarget(null);
      await loadPlans();
    } catch (error) {
      setErrorMessage(error instanceof Error ? error.message : 'Failed to delete plan.');
    } finally {
      setDeletePending(false);
    }
  };

  const openEdit = (plan: Plan) => {
    const f = plan.features || {};
    const quota = f.quota || {};
    const pricing = quota.pricing?.default || {};
    setEditState({
      open: true,
      planId: plan.id,
      data: {
        name: plan.name,
        slug: plan.slug,
        requests_per_day: plan.requests_per_day?.toString() || '',
        requests_per_month: plan.requests_per_month?.toString() || '',
        daily_credits: quota.daily_credits?.toString() || '',
        monthly_credits: quota.monthly_credits?.toString() || '',
        max_input_tokens_per_request: quota.max_input_tokens_per_request?.toString() || '',
        max_output_tokens_per_request: quota.max_output_tokens_per_request?.toString() || '',
        input_per_token: pricing.input_per_token?.toString() || '1',
        output_per_token: pricing.output_per_token?.toString() || '4',
        cached_input_per_token: pricing.cached_input_per_token?.toString() || '0.2',
        price_usd: plan.price_usd || '',
        price_vnd: plan.price_vnd?.toString() || '',
        sort_order: plan.sort_order.toString(),
        active: plan.active,
        streaming: f.streaming !== false,
        priority: !!f.priority,
        dedicated_support: !!f.dedicated_support,
        rate_limit: (f.rate_limit as string) || '10/min',
        model_limits: Object.fromEntries(
          Object.entries(f.model_limits || {}).map(([slug, value]) => {
            const limits = value as Record<string, unknown>;
            return [slug, {
              chat_per_day: limits.chat_per_day?.toString() || '',
              image_per_day: limits.image_per_day?.toString() || '',
              video_per_day: limits.video_per_day?.toString() || '',
            }];
          })
        ),
      },
    });
  };

  const filteredPlans = useMemo(() => {
    const keyword = search.trim().toLowerCase();
    return [...plans]
      .sort((a, b) => a.sort_order - b.sort_order)
      .filter((plan) => {
        if (statusFilter === 'active' && !plan.active) return false;
        if (statusFilter === 'inactive' && plan.active) return false;
        if (!keyword) return true;
        return plan.name.toLowerCase().includes(keyword) || plan.slug.toLowerCase().includes(keyword);
      });
  }, [plans, search, statusFilter]);

  const paginatedPlans = useMemo(() => {
    const start = (page - 1) * pageSize;
    return filteredPlans.slice(start, start + pageSize);
  }, [filteredPlans, page, pageSize]);

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h1 className="text-3xl font-bold">Plans</h1>
        <Button onClick={() => setCreateOpen(true)}>
          <Plus className="h-4 w-4 mr-2" />Create Plan
        </Button>
      </div>

      <Card>
        <CardHeader><CardTitle>Plan List</CardTitle></CardHeader>
        <CardContent>
          <AdminTableToolbar
            searchValue={search}
            onSearchChange={setSearch}
            searchPlaceholder="Search plan name or slug"
            summary={`${filteredPlans.length} plans`}
            actions={(
              <Button variant="outline" size="sm" onClick={() => setStatusFilter((current) => current === 'all' ? 'active' : current === 'active' ? 'inactive' : 'all')}>
                {statusFilter === 'all' ? 'All statuses' : statusFilter === 'active' ? 'Active only' : 'Inactive only'}
              </Button>
            )}
          />
          {errorMessage && (
            <div className="mb-4 rounded-lg border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-200">
              {errorMessage}
            </div>
          )}
          {loading ? (
            <div className="text-center py-8 text-slate-400">Loading...</div>
          ) : (
            <>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Name</TableHead>
                    <TableHead>Slug</TableHead>
                    <TableHead>Price</TableHead>
                    <TableHead>Credits</TableHead>
                    <TableHead>Token Guard</TableHead>
                    <TableHead>Request Fallback</TableHead>
                    <TableHead>Features</TableHead>
                    <TableHead>Status</TableHead>
                    <TableHead>Actions</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {paginatedPlans.length ? paginatedPlans.map((plan) => (
                    <TableRow key={plan.id}>
                      <TableCell className="font-medium">{plan.name}</TableCell>
                      <TableCell className="font-mono text-sm">{plan.slug}</TableCell>
                      <TableCell>
                        {plan.price_usd ? `$${plan.price_usd}` : 'Free'}
                        {plan.price_vnd ? ` (${Number(plan.price_vnd).toLocaleString()}d)` : ''}
                      </TableCell>
                      <TableCell className="text-sm text-slate-500">
                        {describeCredits(plan.features)}
                      </TableCell>
                      <TableCell className="text-sm text-slate-500">
                        {describeTokenGuard(plan.features)}
                      </TableCell>
                      <TableCell className="text-sm text-slate-500">
                        D {formatLimit(plan.requests_per_day)} · M {formatLimit(plan.requests_per_month)}
                      </TableCell>
                      <TableCell>
                        <div className="flex flex-wrap gap-1">
                          {plan.features?.streaming && <Badge variant="outline" className="text-xs">Stream</Badge>}
                          {plan.features?.priority && <Badge variant="outline" className="text-xs">Priority</Badge>}
                          {plan.features?.dedicated_support && <Badge variant="outline" className="text-xs">Support</Badge>}
                          {plan.features?.rate_limit && <Badge variant="secondary" className="text-xs">{plan.features.rate_limit as string}</Badge>}
                        </div>
                      </TableCell>
                      <TableCell>
                        <Button size="sm" variant={plan.active ? 'default' : 'outline'} onClick={() => handleToggleActive(plan.id, plan.active)}>
                          {plan.active ? <><Check className="h-3 w-3 mr-1" />Active</> : <><X className="h-3 w-3 mr-1" />Inactive</>}
                        </Button>
                      </TableCell>
                      <TableCell>
                        <div className="flex gap-2">
                          <Button size="sm" variant="ghost" onClick={() => openEdit(plan)}>
                            <Edit className="h-4 w-4" />
                          </Button>
                          <Button size="sm" variant="ghost" onClick={() => setDeleteTarget(plan)}>
                            <Trash2 className="h-4 w-4 text-red-500" />
                          </Button>
                        </div>
                      </TableCell>
                    </TableRow>
                  )) : (
                    <TableRow>
                      <TableCell colSpan={8} className="py-8 text-center text-sm text-slate-500">
                        No plans match the current filters.
                      </TableCell>
                    </TableRow>
                  )}
                </TableBody>
              </Table>
              <AdminTablePagination
                page={page}
                pageSize={pageSize}
                visibleCount={paginatedPlans.length}
                totalCount={filteredPlans.length}
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

      <PlanFormDialog
        open={createOpen}
        onClose={() => setCreateOpen(false)}
        onSaved={loadPlans}
        title="Create New Plan"
        description="Add a new subscription plan"
      />

      <PlanFormDialog
        open={editState.open}
        onClose={() => setEditState({ open: false, planId: null, data: {} })}
        onSaved={loadPlans}
        editPlanId={editState.planId}
        initialData={editState.data}
        title="Edit Plan"
        description="Update plan settings and allowed models"
      />

      <ConfirmActionDialog
        open={!!deleteTarget}
        onOpenChange={(open) => !open && setDeleteTarget(null)}
        title="Delete plan?"
        description={`Plan ${deleteTarget?.name || ''} will be removed. Plans assigned to users cannot be deleted.`}
        confirmLabel="Delete Plan"
        loading={deletePending}
        onConfirm={handleDelete}
      />
    </div>
  );
}
