'use client';

import { type ReactNode, useEffect, useState } from 'react';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import {
  Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Switch } from '@/components/ui/switch';
import { adminFetch } from '@/lib/api';

interface Model {
  id: number;
  name: string;
  slug: string;
  provider_name: string;
}

interface ModelLimitDraft {
  chat_per_day: string;
  image_per_day: string;
  video_per_day: string;
}

export interface PlanFormData {
  name: string;
  slug: string;
  requests_per_day: string;
  requests_per_month: string;
  price_usd: string;
  price_vnd: string;
  sort_order: string;
  active: boolean;
  streaming: boolean;
  priority: boolean;
  dedicated_support: boolean;
  rate_limit: string;
  selected_model_ids: number[];
  model_limits: Record<string, ModelLimitDraft>;
}

interface Props {
  open: boolean;
  onClose: () => void;
  onSaved: () => void;
  editPlanId?: number | null;
  initialData?: Partial<PlanFormData>;
  title: string;
  description: string;
}

type TextFieldKey =
  | 'name'
  | 'slug'
  | 'price_usd'
  | 'price_vnd'
  | 'requests_per_day'
  | 'requests_per_month'
  | 'sort_order'
  | 'rate_limit';

type ToggleFieldKey = 'active' | 'streaming' | 'priority' | 'dedicated_support';

interface TextFieldConfig {
  key: TextFieldKey;
  label: string;
  placeholder?: string;
  type?: 'number' | 'text';
  helper?: string;
  hiddenWhenEditing?: boolean;
}

interface ToggleConfig {
  key: ToggleFieldKey;
  label: string;
  description: string;
}

const identityFields: TextFieldConfig[] = [
  { key: 'name', label: 'Plan name', placeholder: 'Pro' },
  { key: 'slug', label: 'Slug', placeholder: 'pro', hiddenWhenEditing: true },
];

const pricingAndQuotaFields: TextFieldConfig[] = [
  { key: 'price_usd', label: 'Price USD', type: 'number', placeholder: '0 for free' },
  { key: 'price_vnd', label: 'Price VND', type: 'number', placeholder: 'Optional' },
  { key: 'requests_per_day', label: 'Requests / day', type: 'number', placeholder: '-1', helper: '-1 means unlimited' },
  { key: 'requests_per_month', label: 'Requests / month', type: 'number', placeholder: '-1', helper: '-1 means unlimited' },
  { key: 'sort_order', label: 'Sort order', type: 'number', placeholder: '0' },
  { key: 'rate_limit', label: 'Rate limit', placeholder: '10/min', helper: 'Shown in plan features and enforced by backend' },
];

const toggleFields: ToggleConfig[] = [
  { key: 'active', label: 'Active', description: 'Visible to customers and available immediately.' },
  { key: 'streaming', label: 'Streaming', description: 'Enable streamed model responses for this plan.' },
  { key: 'priority', label: 'Priority', description: 'Prefer faster handling for subscribed users.' },
  { key: 'dedicated_support', label: 'Support', description: 'Show premium support as part of this plan.' },
];

export const defaultPlanForm: PlanFormData = {
  name: '', slug: '', requests_per_day: '', requests_per_month: '',
  price_usd: '', price_vnd: '', sort_order: '0', active: true,
  streaming: true, priority: false, dedicated_support: false,
  rate_limit: '10/min', selected_model_ids: [], model_limits: {},
};

const emptyModelLimitDraft = (): ModelLimitDraft => ({
  chat_per_day: '',
  image_per_day: '',
  video_per_day: '',
});

function SectionCard({
  title, description, children, className = '',
}: { title: string; description: string; children: ReactNode; className?: string }) {
  return (
    <section className={`rounded-[var(--radius)] border bg-card/70 ${className}`}>
      <div className="border-b px-4 py-3">
        <h3 className="text-sm font-semibold text-foreground">{title}</h3>
        <p className="mt-1 text-xs text-muted-foreground">{description}</p>
      </div>
      <div className="px-4 py-4">{children}</div>
    </section>
  );
}

export function PlanFormDialog({ open, onClose, onSaved, editPlanId, initialData, title, description }: Props) {
  const [form, setForm] = useState<PlanFormData>({ ...defaultPlanForm, ...initialData });
  const [allModels, setAllModels] = useState<Model[]>([]);
  const [loading, setLoading] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    setForm({ ...defaultPlanForm, ...initialData });
    setFormError(null);
    loadModels();
    if (editPlanId) loadPlanModels(editPlanId);
  }, [open, editPlanId, initialData]);

  const loadModels = async () => {
    try {
      const res = await adminFetch<{ data: Model[] }>('/admin/models');
      setAllModels(res.data || []);
    } catch (error) {
      setFormError(error instanceof Error ? error.message : 'Failed to load models.');
    }
  };

  const loadPlanModels = async (planId: number) => {
    try {
      const res = await adminFetch<{ data: { id: number }[] }>(`/admin/plans/${planId}/models`);
      setForm((prev) => ({ ...prev, selected_model_ids: (res.data || []).map((m) => m.id) }));
    } catch (error) {
      setFormError(error instanceof Error ? error.message : 'Failed to load plan models.');
    }
  };

  const grouped = allModels.reduce<Record<string, Model[]>>((acc, model) => {
    const provider = model.provider_name || 'Other';
    if (!acc[provider]) acc[provider] = [];
    acc[provider].push(model);
    return acc;
  }, {});

  const selectedModelsCount = form.selected_model_ids.length;
  const selectedModels = allModels.filter((model) => form.selected_model_ids.includes(model.id));
  const set = (key: keyof PlanFormData, value: string | boolean | number) =>
    setForm((prev) => ({ ...prev, [key]: value }));

  const toggleModel = (modelId: number) => {
    const model = allModels.find((item) => item.id === modelId);
    setForm((prev) => {
      const selected = prev.selected_model_ids.includes(modelId);
      const nextSelectedIds = selected
        ? prev.selected_model_ids.filter((id) => id !== modelId)
        : [...prev.selected_model_ids, modelId];
      const nextModelLimits = { ...prev.model_limits };

      if (selected && model) {
        delete nextModelLimits[model.slug];
      } else if (!selected && model && !nextModelLimits[model.slug]) {
        nextModelLimits[model.slug] = emptyModelLimitDraft();
      }

      return {
        ...prev,
        selected_model_ids: nextSelectedIds,
        model_limits: nextModelLimits,
      };
    });
  };

  const toggleProviderModels = (models: Model[]) => {
    setForm((prev) => {
      const providerIds = models.map((model) => model.id);
      const providerIdSet = new Set(providerIds);
      const allSelected = providerIds.every((modelId) => prev.selected_model_ids.includes(modelId));
      const nextSelectedIds = allSelected
        ? prev.selected_model_ids.filter((id) => !providerIdSet.has(id))
        : Array.from(new Set([...prev.selected_model_ids, ...providerIds]));
      const nextModelLimits = { ...prev.model_limits };

      if (allSelected) {
        models.forEach((model) => {
          delete nextModelLimits[model.slug];
        });
      } else {
        models.forEach((model) => {
          if (!nextModelLimits[model.slug]) {
            nextModelLimits[model.slug] = emptyModelLimitDraft();
          }
        });
      }

      return {
        ...prev,
        selected_model_ids: nextSelectedIds,
        model_limits: nextModelLimits,
      };
    });
  };

  const setModelLimit = (
    modelSlug: string,
    key: keyof ModelLimitDraft,
    value: string,
  ) => {
    setForm((prev) => ({
      ...prev,
      model_limits: {
        ...prev.model_limits,
        [modelSlug]: {
          ...(prev.model_limits[modelSlug] || emptyModelLimitDraft()),
          [key]: value,
        },
      },
    }));
  };

  const buildModelLimitsPayload = () => {
    const entries = selectedModels
      .map((model) => {
        const draft = form.model_limits[model.slug];
        if (!draft) return null;

        const parsed = Object.entries(draft).reduce<Record<string, number>>((acc, [key, value]) => {
          const trimmed = value.trim();
          if (!trimmed) {
            return acc;
          }

          const numericValue = Number.parseInt(trimmed, 10);
          if (!Number.isNaN(numericValue)) {
            acc[key] = numericValue;
          }
          return acc;
        }, {});

        return Object.keys(parsed).length ? [model.slug, parsed] : null;
      })
      .filter(Boolean);

    return Object.fromEntries(entries as [string, Record<string, number>][]);
  };

  const handleSave = async () => {
    setLoading(true);
    try {
      setFormError(null);
      const payload = {
        name: form.name,
        slug: form.slug,
        requests_per_day: form.requests_per_day ? parseInt(form.requests_per_day) : null,
        requests_per_month: form.requests_per_month ? parseInt(form.requests_per_month) : null,
        price_usd: form.price_usd.trim() ? form.price_usd.trim() : null,
        price_vnd: form.price_vnd ? parseInt(form.price_vnd) : null,
        features: {
          streaming: form.streaming,
          priority: form.priority,
          dedicated_support: form.dedicated_support,
          rate_limit: form.rate_limit,
          model_limits: buildModelLimitsPayload(),
        },
        active: form.active,
        sort_order: parseInt(form.sort_order) || 0,
      };

      let planId = editPlanId;
      if (editPlanId) {
        await adminFetch(`/admin/plans/${editPlanId}`, {
          method: 'PUT',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(payload),
        });
      } else {
        const created = await adminFetch<{ id: number }>('/admin/plans', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(payload),
        });
        planId = created?.id;
      }

      if (planId) {
        await adminFetch(`/admin/plans/${planId}/models`, {
          method: 'PUT',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ model_ids: form.selected_model_ids }),
        });
      }

      onSaved();
      onClose();
    } catch (error) {
      setFormError(error instanceof Error ? error.message : 'Failed to save plan.');
    }
    finally { setLoading(false); }
  };

  return (
    <Dialog open={open} onOpenChange={(nextOpen) => !nextOpen && onClose()}>
      <DialogContent className="w-[calc(100vw-2rem)] !max-w-[min(96vw,60rem)] sm:!max-w-[min(94vw,60rem)] overflow-hidden p-0">
        <DialogHeader className="border-b bg-muted/20 px-5 py-4 sm:px-6">
          <div className="flex items-start justify-between gap-3 pr-8">
            <div>
              <DialogTitle>{title}</DialogTitle>
              <DialogDescription className="mt-1">{description}</DialogDescription>
            </div>
            <div className="flex flex-wrap justify-end gap-2">
              <Badge variant="outline">{selectedModelsCount} models</Badge>
              <Badge variant={form.active ? 'secondary' : 'outline'}>{form.active ? 'Active' : 'Draft'}</Badge>
            </div>
          </div>
        </DialogHeader>

        <div className="max-h-[calc(90vh-8.5rem)] overflow-y-auto bg-background px-5 py-5 sm:px-6">
          <div className="space-y-4">
            {formError && (
              <div className="border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
                {formError}
              </div>
            )}
            <div className="space-y-4">
              <SectionCard title="Identity" description="Core plan information customers and admins will recognize.">
                <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
                  {identityFields.map((field) => {
                    if (field.hiddenWhenEditing && editPlanId) return null;
                    const inputId = `plan-${field.key}`;
                    return (
                      <div key={field.key} className="space-y-1.5">
                        <label htmlFor={inputId} className="text-sm font-medium text-foreground">{field.label}</label>
                        <Input
                          id={inputId}
                          type={field.type}
                          value={form[field.key]}
                          onChange={(e) => set(field.key, e.target.value)}
                          placeholder={field.placeholder}
                        />
                      </div>
                    );
                  })}
                </div>
              </SectionCard>

              <SectionCard title="Behavior" description="Feature flags that change how this plan behaves in the product.">
                <div className="grid gap-2 sm:grid-cols-2 xl:grid-cols-4">
                  {toggleFields.map((item) => (
                    <label
                      key={item.key}
                      className="grid min-h-14 cursor-pointer grid-cols-[minmax(0,1fr)_auto] items-center gap-3 rounded-[var(--radius)] border bg-background px-3 py-2.5 transition-colors hover:bg-muted/40"
                    >
                      <span className="min-w-0">
                        <span className="block text-sm font-medium text-foreground">{item.label}</span>
                        <span className="block text-xs text-muted-foreground">{item.description}</span>
                      </span>
                      <Switch checked={form[item.key]} onCheckedChange={(value) => set(item.key, value)} />
                    </label>
                  ))}
                </div>
              </SectionCard>
            </div>

            <SectionCard title="Pricing & Quotas" description="Billing, request caps, and API throughput settings.">
              <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3">
                {pricingAndQuotaFields.map((field) => {
                  const inputId = `plan-${field.key}`;
                  return (
                    <div key={field.key} className="space-y-1.5">
                      <label htmlFor={inputId} className="text-sm font-medium text-foreground">{field.label}</label>
                      <Input
                        id={inputId}
                        type={field.type}
                        value={form[field.key]}
                        onChange={(e) => set(field.key, e.target.value)}
                        placeholder={field.placeholder}
                      />
                      {field.helper && <p className="text-xs text-muted-foreground">{field.helper}</p>}
                    </div>
                  );
                })}
              </div>
            </SectionCard>

            <SectionCard title="Allowed Models" description="Keep selection tight so each plan has a clear and predictable scope.">
              <div className="mb-3 flex flex-wrap items-center gap-2">
                <Badge variant="outline">{Object.keys(grouped).length || 0} providers</Badge>
                <Badge variant="outline">{selectedModelsCount} selected</Badge>
              </div>

              {allModels.length === 0 && (
                <div className="rounded-[var(--radius)] border border-dashed px-4 py-6 text-sm text-muted-foreground">
                  No models found.
                </div>
              )}

              <div className="space-y-3">
                {Object.entries(grouped).map(([provider, models]) => {
                  const selectedInProvider = models.filter((model) => form.selected_model_ids.includes(model.id)).length;
                  const providerChecked = models.length > 0 && selectedInProvider === models.length;

                  return (
                  <section key={provider} className="overflow-hidden rounded-[var(--radius)] border bg-background/80">
                    <div className="flex items-center justify-between border-b px-3 py-2">
                      <label className="flex cursor-pointer items-center gap-2">
                        <Checkbox
                          checked={providerChecked}
                          onCheckedChange={() => toggleProviderModels(models)}
                          className="border-slate-600 data-[checked]:border-sky-500 data-[checked]:bg-sky-500"
                        />
                        <span className="text-xs font-semibold uppercase tracking-[0.16em] text-muted-foreground">{provider}</span>
                      </label>
                      <Badge variant="secondary">{selectedInProvider}/{models.length}</Badge>
                    </div>
                    <div className="grid grid-cols-1 gap-x-2 gap-y-1.5 px-3 py-3 sm:grid-cols-2 lg:grid-cols-3">
                      {models.map((model) => (
                        <label
                          key={model.id}
                          title={model.slug}
                          className="grid min-h-11 cursor-pointer grid-cols-[1rem_minmax(0,1fr)] items-center gap-2 rounded-[var(--radius)] px-2 py-1.5 transition-colors hover:bg-muted/50"
                        >
                          <Checkbox
                            checked={form.selected_model_ids.includes(model.id)}
                            onCheckedChange={() => toggleModel(model.id)}
                            className="border-slate-600 data-[checked]:border-sky-500 data-[checked]:bg-sky-500"
                          />
                          <span className="truncate font-mono text-sm font-medium text-foreground">{model.slug}</span>
                        </label>
                      ))}
                    </div>
                  </section>
                )})}
              </div>
            </SectionCard>

            <SectionCard
              title="Per-model limits"
              description="Optional daily limits for each selected model and request type. Leave blank to fall back to the plan-wide limit."
            >
              {!selectedModels.length ? (
                <div className="rounded-[var(--radius)] border border-dashed px-4 py-6 text-sm text-muted-foreground">
                  Select at least one model to configure custom limits.
                </div>
              ) : (
                <div className="space-y-3">
                  {selectedModels.map((model) => {
                    const draft = form.model_limits[model.slug] || emptyModelLimitDraft();
                    return (
                      <section key={model.id} className="rounded-[var(--radius)] border bg-background/80 px-4 py-4">
                        <div className="mb-3 flex flex-wrap items-center justify-between gap-2">
                          <div>
                            <div className="font-mono text-sm font-semibold text-foreground">{model.slug}</div>
                            <div className="text-xs text-muted-foreground">{model.name}</div>
                          </div>
                          <Badge variant="outline">{model.provider_name}</Badge>
                        </div>

                        <div className="grid grid-cols-1 gap-3 md:grid-cols-3">
                          {([
                            ['chat_per_day', 'Chat / day'],
                            ['image_per_day', 'Images / day'],
                            ['video_per_day', 'Videos / day'],
                          ] as Array<[keyof ModelLimitDraft, string]>).map(([key, label]) => {
                            const inputId = `plan-limit-${model.slug}-${key}`;
                            return (
                              <div key={key} className="space-y-1.5">
                                <label htmlFor={inputId} className="text-sm font-medium text-foreground">{label}</label>
                                <Input
                                  id={inputId}
                                  type="number"
                                  value={draft[key]}
                                  onChange={(e) => setModelLimit(model.slug, key, e.target.value)}
                                  placeholder="blank = use global limit"
                                />
                              </div>
                            );
                          })}
                        </div>
                      </section>
                    );
                  })}
                </div>
              )}
            </SectionCard>
          </div>
        </div>

        <DialogFooter className="!mx-0 !mb-0 bg-muted/30 px-5 py-4 sm:px-6">
          <Button variant="outline" onClick={onClose}>Cancel</Button>
          <Button onClick={handleSave} disabled={loading}>
            {loading ? 'Saving...' : editPlanId ? 'Save Changes' : 'Create Plan'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
