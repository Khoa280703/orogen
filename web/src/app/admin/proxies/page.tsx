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
import { Switch } from '@/components/ui/switch';
import { listProxies, createProxy, updateProxy, deleteProxy } from '@/lib/api';
import { Plus, Edit, Trash2 } from 'lucide-react';

interface Proxy {
  id: number;
  url: string;
  label: string | null;
  active: boolean;
  created_at: string | null;
  assigned_accounts: number;
}

export default function ProxiesPage() {
  const [proxies, setProxies] = useState<Proxy[]>([]);
  const [loading, setLoading] = useState(true);
  const [pageError, setPageError] = useState<string | null>(null);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editProxy, setEditProxy] = useState<Proxy | null>(null);
  const [url, setUrl] = useState('');
  const [label, setLabel] = useState('');
  const [error, setError] = useState('');
  const [deleteTarget, setDeleteTarget] = useState<Proxy | null>(null);
  const [deletePending, setDeletePending] = useState(false);
  const [togglePendingId, setTogglePendingId] = useState<number | null>(null);
  const [search, setSearch] = useState('');
  const [statusFilter, setStatusFilter] = useState('all');
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);

  useEffect(() => {
    loadProxies();
  }, []);

  useEffect(() => {
    setPage(1);
  }, [search, statusFilter]);

  const loadProxies = async () => {
    try {
      setPageError(null);
      const data = await listProxies();
      setProxies(data);
    } catch (error) {
      setPageError(error instanceof Error ? error.message : 'Failed to load proxies.');
    } finally {
      setLoading(false);
    }
  };

  const validateUrl = (value: string): boolean => {
    if (!value.trim()) return false;
    if (value.length > 500) return false;
    // Validate proxy format: host:port:user:pass
    const proxyPattern = /^[^:]+:\d+:[^:]+:[^:]+$/;
    return proxyPattern.test(value);
  };

  const parseProxyInput = (value: string): string => {
    // Convert host:port:user:pass to socks5h://user:pass@host:port
    const parts = value.split(':');
    if (parts.length === 4) {
      const [host, port, user, pass] = parts;
      return `socks5h://${user}:${pass}@${host}:${port}`;
    }
    return value;
  };

  const validateLabel = (value: string): boolean => {
    if (!value) return true; // Optional field
    if (value.length > 100) return false;
    return /^[a-zA-Z0-9 _-]+$/.test(value);
  };

  const handleCreate = async () => {
    setError('');

    if (!validateUrl(url)) {
      setError('Invalid proxy format. Use: host:port:user:pass (max 500 chars)');
      return;
    }

    if (!validateLabel(label)) {
      setError('Invalid label. Use only letters, numbers, spaces, underscores, hyphens (max 100 chars)');
      return;
    }

    try {
      const proxyUrl = parseProxyInput(url);
      await createProxy(proxyUrl, label || undefined);
      setPageError(null);
      setUrl('');
      setLabel('');
      setDialogOpen(false);
      loadProxies();
    } catch (error) {
      setError(error instanceof Error ? error.message : 'Failed to create proxy.');
    }
  };

  const handleEdit = async () => {
    setError('');

    if (!editProxy) return;

    if (url && !validateUrl(url)) {
      setError('Invalid proxy format. Use: host:port:user:pass (max 500 chars)');
      return;
    }

    if (!validateLabel(label)) {
      setError('Invalid label. Use only letters, numbers, spaces, underscores, hyphens (max 100 chars)');
      return;
    }

    try {
      const proxyUrl = url ? parseProxyInput(url) : undefined;
      await updateProxy(editProxy.id, {
        url: proxyUrl,
        label: label || undefined,
      });
      setPageError(null);
      setEditProxy(null);
      setUrl('');
      setLabel('');
      setDialogOpen(false);
      loadProxies();
    } catch (error) {
      setError(error instanceof Error ? error.message : 'Failed to update proxy.');
    }
  };

  const handleDelete = async () => {
    if (!deleteTarget) return;
    setDeletePending(true);
    try {
      await deleteProxy(deleteTarget.id);
      setDeleteTarget(null);
      setPageError(null);
      await loadProxies();
    } catch (error) {
      setPageError(error instanceof Error ? error.message : 'Failed to delete proxy.');
    } finally {
      setDeletePending(false);
    }
  };

  const handleToggleActive = async (proxy: Proxy, nextActive: boolean) => {
    setTogglePendingId(proxy.id);
    setPageError(null);
    try {
      await updateProxy(proxy.id, { active: nextActive });
      setProxies((current) =>
        current.map((item) =>
          item.id === proxy.id ? { ...item, active: nextActive } : item
        )
      );
    } catch (error) {
      setPageError(error instanceof Error ? error.message : 'Failed to update proxy status.');
    } finally {
      setTogglePendingId(null);
    }
  };

  const openEditDialog = (proxy: Proxy) => {
    setEditProxy(proxy);
    setUrl(proxy.url);
    setLabel(proxy.label || '');
    setError('');
    setDialogOpen(true);
  };

  const openCreateDialog = () => {
    setEditProxy(null);
    setUrl('');
    setLabel('');
    setError('');
    setDialogOpen(true);
  };

  const filteredProxies = useMemo(() => {
    const keyword = search.trim().toLowerCase();
    return proxies.filter((proxy) => {
      if (statusFilter === 'active' && !proxy.active) return false;
      if (statusFilter === 'inactive' && proxy.active) return false;
      if (!keyword) return true;
      return (
        (proxy.label || '').toLowerCase().includes(keyword) ||
        proxy.url.toLowerCase().includes(keyword)
      );
    });
  }, [proxies, search, statusFilter]);

  const paginatedProxies = useMemo(() => {
    const start = (page - 1) * pageSize;
    return filteredProxies.slice(start, start + pageSize);
  }, [filteredProxies, page, pageSize]);

  if (loading) {
    return <div className="text-slate-400">Loading...</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h1 className="text-3xl font-bold">Proxies</h1>
        <Button onClick={openCreateDialog}>
          <Plus className="w-4 h-4 mr-2" />
          Add Proxy
        </Button>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Proxy List</CardTitle>
          <CardDescription>
            Manage SOCKS5 proxies for Grok API requests
          </CardDescription>
        </CardHeader>
        <CardContent>
          <AdminTableToolbar
            searchValue={search}
            onSearchChange={setSearch}
            searchPlaceholder="Search label or URL"
            summary={`${filteredProxies.length} proxies`}
            actions={(
              <Select value={statusFilter} onValueChange={(value) => setStatusFilter(value || 'all')}>
                <SelectTrigger className="w-40">
                  <SelectValue placeholder="Filter by status" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All statuses</SelectItem>
                  <SelectItem value="active">Active</SelectItem>
                  <SelectItem value="inactive">Inactive</SelectItem>
                </SelectContent>
              </Select>
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
                <TableHead>URL</TableHead>
                <TableHead>Active</TableHead>
                <TableHead>Assigned</TableHead>
                <TableHead>Created</TableHead>
                <TableHead className="w-32">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {paginatedProxies.length ? paginatedProxies.map((proxy) => (
                <TableRow key={proxy.id}>
                  <TableCell>
                    {proxy.label || <span className="text-slate-500">-</span>}
                  </TableCell>
                  <TableCell className="font-mono text-sm">
                    {proxy.url.split('@')[1] || proxy.url}
                  </TableCell>
                  <TableCell>
                    <div className="flex items-center gap-3">
                      <Switch
                        checked={proxy.active}
                        disabled={togglePendingId === proxy.id}
                        onCheckedChange={(checked) => handleToggleActive(proxy, checked)}
                        aria-label={`${proxy.active ? 'Deactivate' : 'Activate'} proxy ${proxy.label || proxy.id}`}
                      />
                      <Badge variant={proxy.active ? 'default' : 'secondary'}>
                        {proxy.active ? 'Active' : 'Inactive'}
                      </Badge>
                    </div>
                  </TableCell>
                  <TableCell>{proxy.assigned_accounts}</TableCell>
                  <TableCell className="text-slate-400">
                    {proxy.created_at?.slice(0, 10) || '-'}
                  </TableCell>
                  <TableCell>
                    <div className="flex gap-2">
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => openEditDialog(proxy)}
                      >
                        <Edit className="w-4 h-4" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => setDeleteTarget(proxy)}
                      >
                        <Trash2 className="w-4 h-4 text-red-500" />
                      </Button>
                    </div>
                  </TableCell>
                </TableRow>
              )) : (
                <TableRow>
                  <TableCell colSpan={6} className="py-8 text-center text-sm text-slate-500">
                    No proxies match the current filters.
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          </Table>
          <AdminTablePagination
            page={page}
            pageSize={pageSize}
            visibleCount={paginatedProxies.length}
            totalCount={filteredProxies.length}
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
            <DialogTitle>
              {editProxy ? 'Edit Proxy' : 'Add Proxy'}
            </DialogTitle>
            <DialogDescription>
              Enter proxy details (format: host:port:user:pass)
            </DialogDescription>
          </DialogHeader>
          {error && (
            <div className="p-3 bg-red-500/10 border border-red-500/20 rounded text-red-400 text-sm">
              {error}
            </div>
          )}
          <div className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">URL</label>
              <Input
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                placeholder="149.19.197.214:51845:khoa2807:khoa2807"
                maxLength={500}
              />
              <p className="text-xs text-slate-500">
                Format: host:port:user:pass (max 500 chars)
              </p>
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">Label (optional)</label>
              <Input
                value={label}
                onChange={(e) => setLabel(e.target.value)}
                placeholder="US Proxy 1"
                maxLength={100}
              />
              <p className="text-xs text-slate-500">
                Letters, numbers, spaces, underscores, hyphens (max 100 chars)
              </p>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDialogOpen(false)}>
              Cancel
            </Button>
            <Button onClick={editProxy ? handleEdit : handleCreate}>
              {editProxy ? 'Save' : 'Create'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <ConfirmActionDialog
        open={!!deleteTarget}
        onOpenChange={(open) => !open && setDeleteTarget(null)}
        title="Delete proxy?"
        description={`Proxy ${deleteTarget?.label || deleteTarget?.url || ''} will be removed from the system.`}
        confirmLabel="Delete Proxy"
        loading={deletePending}
        onConfirm={handleDelete}
      />
    </div>
  );
}
