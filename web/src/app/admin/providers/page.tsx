'use client';

import { useEffect, useMemo, useState } from 'react';
import { Edit, Plus, Power, Trash2 } from 'lucide-react';
import { AdminTablePagination } from '@/components/admin/admin-table-pagination';
import { AdminTableToolbar } from '@/components/admin/admin-table-toolbar';
import { ConfirmActionDialog } from '@/components/confirm-action-dialog';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import {
  deactivateProviderModel,
  listProviderModels,
  listProviders,
  updateProvider,
  updateProviderModel,
  type ProviderModelSummary,
  type ProviderSummary,
} from '@/lib/api';
import { ModelFormDialog } from '../models/model-form-dialog';
import { ProviderFormDialog } from './provider-form-dialog';

export default function ProvidersPage() {
  const [providers, setProviders] = useState<ProviderSummary[]>([]);
  const [models, setModels] = useState<ProviderModelSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [providerDialogOpen, setProviderDialogOpen] = useState(false);
  const [modelDialogOpen, setModelDialogOpen] = useState(false);
  const [editingProvider, setEditingProvider] = useState<ProviderSummary | null>(null);
  const [editingModel, setEditingModel] = useState<ProviderModelSummary | null>(null);
  const [scopedProvider, setScopedProvider] = useState<ProviderSummary | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<ProviderModelSummary | null>(null);
  const [deletePending, setDeletePending] = useState(false);
  const [search, setSearch] = useState('');
  const [statusFilter, setStatusFilter] = useState('all');
  const [selectedProviderId, setSelectedProviderId] = useState<number | null>(null);
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(10);

  const loadData = async () => {
    try {
      setErrorMessage(null);
      const [providersResponse, modelsResponse] = await Promise.all([listProviders(), listProviderModels()]);
      setProviders(providersResponse.data);
      setModels(modelsResponse.data);
    } catch (loadError) {
      setErrorMessage(loadError instanceof Error ? loadError.message : 'Failed to load provider catalog.');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void loadData();
  }, []);

  useEffect(() => {
    setPage(1);
  }, [search, statusFilter]);

  const modelsByProvider = useMemo(() => {
    return models.reduce<Record<number, ProviderModelSummary[]>>((accumulator, model) => {
      const bucket = accumulator[model.provider_id] || [];
      bucket.push(model);
      accumulator[model.provider_id] = bucket;
      return accumulator;
    }, {});
  }, [models]);

  const filteredProviders = useMemo(() => {
    const keyword = search.trim().toLowerCase();
    return [...providers]
      .sort((left, right) => left.name.localeCompare(right.name))
      .filter((provider) => {
        if (statusFilter === 'active' && !provider.active) return false;
        if (statusFilter === 'inactive' && provider.active) return false;
        if (!keyword) return true;
        const matchesProvider =
          provider.name.toLowerCase().includes(keyword) || provider.slug.toLowerCase().includes(keyword);
        if (matchesProvider) return true;
        return (modelsByProvider[provider.id] || []).some((model) =>
          [model.name, model.slug, model.description || ''].some((value) => value.toLowerCase().includes(keyword))
        );
      });
  }, [modelsByProvider, providers, search, statusFilter]);

  const paginatedProviders = useMemo(() => {
    const start = (page - 1) * pageSize;
    return filteredProviders.slice(start, start + pageSize);
  }, [filteredProviders, page, pageSize]);

  useEffect(() => {
    if (!paginatedProviders.length) {
      setSelectedProviderId(null);
      return;
    }

    setSelectedProviderId((current) => (
      current && paginatedProviders.some((provider) => provider.id === current)
        ? current
        : paginatedProviders[0].id
    ));
  }, [paginatedProviders]);

  const activeProvider = useMemo(() => {
    if (!selectedProviderId) return null;
    return paginatedProviders.find((provider) => provider.id === selectedProviderId) || null;
  }, [paginatedProviders, selectedProviderId]);

  const activeProviderModels = useMemo(() => {
    if (!activeProvider) return [];
    return [...(modelsByProvider[activeProvider.id] || [])].sort(
      (left, right) => left.sort_order - right.sort_order || left.name.localeCompare(right.name)
    );
  }, [activeProvider, modelsByProvider]);

  const handleToggleProvider = async (provider: ProviderSummary) => {
    try {
      setErrorMessage(null);
      await updateProvider(provider.id, { active: !provider.active });
      await loadData();
    } catch (updateError) {
      setErrorMessage(updateError instanceof Error ? updateError.message : 'Failed to update provider.');
    }
  };

  const handleToggleModel = async (model: ProviderModelSummary) => {
    try {
      setErrorMessage(null);
      await updateProviderModel(model.id, { active: !model.active });
      await loadData();
    } catch (updateError) {
      setErrorMessage(updateError instanceof Error ? updateError.message : 'Failed to update model.');
    }
  };

  const handleDeactivateModel = async () => {
    if (!deleteTarget) return;
    setDeletePending(true);
    try {
      await deactivateProviderModel(deleteTarget.id);
      setDeleteTarget(null);
      await loadData();
    } catch (deleteError) {
      setErrorMessage(deleteError instanceof Error ? deleteError.message : 'Failed to deactivate model.');
    } finally {
      setDeletePending(false);
    }
  };

  const openCreateModelDialog = (provider: ProviderSummary) => {
    setEditingModel(null);
    setScopedProvider(provider);
    setModelDialogOpen(true);
  };

  const openEditModelDialog = (model: ProviderModelSummary) => {
    setEditingModel(model);
    setScopedProvider(providers.find((provider) => provider.id === model.provider_id) || null);
    setModelDialogOpen(true);
  };

  if (loading) {
    return <div className="py-10 text-sm text-slate-500">Loading provider catalog...</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Providers</h1>
          <p className="mt-2 text-sm text-slate-500">Config provider và model chung một chỗ để route catalog không bị tách màn quản trị.</p>
        </div>
        <Button
          onClick={() => {
            setEditingProvider(null);
            setProviderDialogOpen(true);
          }}
        >
          <Plus className="mr-2 h-4 w-4" />
          Add Provider
        </Button>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Provider Catalog</CardTitle>
          <CardDescription>Mỗi provider quản lý trực tiếp model catalog của riêng nó, không còn màn model riêng tách rời.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <AdminTableToolbar
            searchValue={search}
            onSearchChange={setSearch}
            searchPlaceholder="Search provider, model, slug, or description"
            summary={`${filteredProviders.length} providers • ${models.length} models`}
            actions={(
              <Button
                variant="outline"
                size="sm"
                onClick={() =>
                  setStatusFilter((current) =>
                    current === 'all' ? 'active' : current === 'active' ? 'inactive' : 'all'
                  )
                }
              >
                {statusFilter === 'all' ? 'All statuses' : statusFilter === 'active' ? 'Active only' : 'Inactive only'}
              </Button>
            )}
          />

          {errorMessage && (
            <div className="border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
              {errorMessage}
            </div>
          )}

          <div className="space-y-4">
            {paginatedProviders.length ? (
              <>
                <div className="overflow-x-auto">
                  <div className="inline-flex min-w-full gap-2 rounded-lg border border-slate-200 bg-slate-50 p-2">
                    {paginatedProviders.map((provider) => {
                      const providerModels = modelsByProvider[provider.id] || [];
                      const isActiveTab = provider.id === selectedProviderId;

                      return (
                        <button
                          key={provider.id}
                          type="button"
                          onClick={() => setSelectedProviderId(provider.id)}
                          className={`min-w-[180px] flex-1 rounded-md border px-4 py-3 text-left transition-colors ${
                            isActiveTab
                              ? 'border-slate-900 bg-white text-slate-950 shadow-sm'
                              : 'border-transparent bg-transparent text-slate-600 hover:border-slate-200 hover:bg-white'
                          }`}
                        >
                          <div className="flex items-center justify-between gap-3">
                            <span className="truncate text-sm font-semibold">{provider.name}</span>
                            <span className={`h-2 w-2 rounded-full ${provider.active ? 'bg-emerald-500' : 'bg-slate-300'}`} />
                          </div>
                          <div className="mt-2 flex items-center gap-2 text-xs text-slate-500">
                            <span className="font-mono">{provider.slug}</span>
                            <span>{providerModels.length} models</span>
                          </div>
                        </button>
                      );
                    })}
                  </div>
                </div>

                {activeProvider ? (
                  <section className="overflow-hidden rounded-lg border border-slate-200 bg-white">
                    <div className="flex flex-col gap-3 border-b border-slate-200 px-4 py-4 lg:flex-row lg:items-start lg:justify-between">
                      <div className="space-y-2">
                        <div className="flex flex-wrap items-center gap-2">
                          <h2 className="text-lg font-semibold text-slate-950">{activeProvider.name}</h2>
                          <Badge variant={activeProvider.active ? 'default' : 'secondary'}>
                            {activeProvider.active ? 'Active' : 'Inactive'}
                          </Badge>
                          <Badge variant="outline">{activeProvider.slug}</Badge>
                          <Badge variant="outline">{activeProviderModels.length} models</Badge>
                        </div>
                        <p className="text-sm text-slate-500">Created {activeProvider.created_at.slice(0, 10)}</p>
                      </div>

                      <div className="flex flex-wrap gap-2">
                        <Button variant="outline" size="sm" onClick={() => openCreateModelDialog(activeProvider)}>
                          <Plus className="mr-2 h-4 w-4" />
                          Add Model
                        </Button>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => {
                            setEditingProvider(activeProvider);
                            setProviderDialogOpen(true);
                          }}
                        >
                          <Edit className="h-4 w-4" />
                        </Button>
                        <Button variant="ghost" size="sm" onClick={() => void handleToggleProvider(activeProvider)}>
                          <Power className="h-4 w-4 text-slate-500" />
                        </Button>
                      </div>
                    </div>

                    <div className="px-4 py-4">
                      {activeProviderModels.length ? (
                        <Table>
                          <TableHeader>
                            <TableRow>
                              <TableHead>Model</TableHead>
                              <TableHead>Slug</TableHead>
                              <TableHead>Order</TableHead>
                              <TableHead>Status</TableHead>
                              <TableHead className="w-36">Actions</TableHead>
                            </TableRow>
                          </TableHeader>
                          <TableBody>
                            {activeProviderModels.map((model) => (
                              <TableRow key={model.id}>
                                <TableCell>
                                  <div className="space-y-1">
                                    <div className="font-medium">{model.name}</div>
                                    {model.description ? (
                                      <p className="line-clamp-2 text-xs text-slate-500">{model.description}</p>
                                    ) : null}
                                  </div>
                                </TableCell>
                                <TableCell className="font-mono text-sm text-slate-500">{model.slug}</TableCell>
                                <TableCell>{model.sort_order}</TableCell>
                                <TableCell>
                                  <Badge variant={model.active ? 'default' : 'secondary'}>
                                    {model.active ? 'Active' : 'Inactive'}
                                  </Badge>
                                </TableCell>
                                <TableCell>
                                  <div className="flex gap-2">
                                    <Button variant="ghost" size="sm" onClick={() => openEditModelDialog(model)}>
                                      <Edit className="h-4 w-4" />
                                    </Button>
                                    <Button variant="ghost" size="sm" onClick={() => void handleToggleModel(model)}>
                                      <Power className="h-4 w-4 text-slate-500" />
                                    </Button>
                                    <Button variant="ghost" size="sm" onClick={() => setDeleteTarget(model)}>
                                      <Trash2 className="h-4 w-4 text-red-500" />
                                    </Button>
                                  </div>
                                </TableCell>
                              </TableRow>
                            ))}
                          </TableBody>
                        </Table>
                      ) : (
                        <div className="rounded-md border border-dashed border-slate-300 px-4 py-5 text-sm text-slate-500">
                          Provider này chưa có model nào trong catalog.
                        </div>
                      )}
                    </div>
                  </section>
                ) : null}
              </>
            ) : (
              <div className="rounded-lg border border-dashed border-slate-300 px-4 py-10 text-center text-sm text-slate-500">
                No providers match the current filters.
              </div>
            )}
          </div>

          <AdminTablePagination
            page={page}
            pageSize={pageSize}
            visibleCount={paginatedProviders.length}
            totalCount={filteredProviders.length}
            onPageChange={setPage}
            onPageSizeChange={(value) => {
              setPageSize(value);
              setPage(1);
            }}
          />
        </CardContent>
      </Card>

      <ProviderFormDialog
        open={providerDialogOpen}
        provider={editingProvider}
        onOpenChange={setProviderDialogOpen}
        onSaved={loadData}
      />

      <ModelFormDialog
        open={modelDialogOpen}
        model={editingModel}
        providers={providers.filter((provider) => provider.active || provider.id === scopedProvider?.id)}
        lockedProvider={scopedProvider}
        onOpenChange={setModelDialogOpen}
        onSaved={loadData}
      />

      <ConfirmActionDialog
        open={Boolean(deleteTarget)}
        onOpenChange={(open) => !open && setDeleteTarget(null)}
        title="Deactivate model?"
        description={`Model ${deleteTarget?.name || ''} will be soft-disabled from the catalog.`}
        confirmLabel="Deactivate"
        loading={deletePending}
        onConfirm={handleDeactivateModel}
      />
    </div>
  );
}
