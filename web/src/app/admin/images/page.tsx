'use client';

import { useEffect, useRef, useState } from 'react';
import { Eye, Trash2 } from 'lucide-react';
import { AdminTablePagination } from '@/components/admin/admin-table-pagination';
import { AdminTableToolbar } from '@/components/admin/admin-table-toolbar';
import { ConfirmActionDialog } from '@/components/confirm-action-dialog';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import {
  deleteAdminImage,
  getAdminImageDetail,
  listAdminImages,
  type AdminImageDetail,
  type AdminImageListItem,
} from '@/lib/api';

const statusOptions = ['all', 'completed', 'failed', 'pending'];
const pageSizeOptions = [10, 20, 50, 100];

export default function AdminImagesPage() {
  const [items, setItems] = useState<AdminImageListItem[]>([]);
  const [total, setTotal] = useState(0);
  const [selected, setSelected] = useState<AdminImageDetail | null>(null);
  const [pendingDeleteId, setPendingDeleteId] = useState<number | null>(null);
  const [deletingId, setDeletingId] = useState<number | null>(null);
  const [search, setSearch] = useState('');
  const [status, setStatus] = useState('all');
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const requestIdRef = useRef(0);

  useEffect(() => {
    setPage(1);
  }, [search, status]);

  useEffect(() => {
    const requestId = ++requestIdRef.current;

    async function load() {
      try {
        setLoading(true);
        setErrorMessage(null);
        const data = await listAdminImages(search, status, pageSize, (page - 1) * pageSize);
        if (requestId !== requestIdRef.current) return;
        setItems(data.items);
        setTotal(data.total);
      } catch (error) {
        if (requestId !== requestIdRef.current) return;
        setErrorMessage(error instanceof Error ? error.message : 'Failed to load image generations.');
      } finally {
        if (requestId !== requestIdRef.current) return;
        setLoading(false);
      }
    }

    void load();
  }, [page, pageSize, search, status]);

  async function handleView(id: number) {
    try {
      setSelected(await getAdminImageDetail(id));
    } catch (error) {
      setErrorMessage(error instanceof Error ? error.message : 'Failed to load image detail.');
    }
  }

  async function handleDelete() {
    if (pendingDeleteId === null) return;

    try {
      setDeletingId(pendingDeleteId);
      setErrorMessage(null);
      await deleteAdminImage(pendingDeleteId);
      setItems((current) => current.filter((item) => item.id !== pendingDeleteId));
      if (selected?.id === pendingDeleteId) setSelected(null);
      setPendingDeleteId(null);
    } catch (error) {
      setErrorMessage(error instanceof Error ? error.message : 'Failed to delete image generation.');
    } finally {
      setDeletingId(null);
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
        <div>
          <h1 className="text-3xl font-bold">Image Generations</h1>
          <p className="mt-1 text-sm text-slate-500">Theo dõi prompt, trạng thái, và asset URLs của user trong media studio.</p>
        </div>
      </div>

      <Card>
        <CardHeader><CardTitle>Generation list</CardTitle></CardHeader>
        <CardContent>
          <AdminTableToolbar
            searchValue={search}
            onSearchChange={setSearch}
            searchPlaceholder="Search user or prompt"
            summary={loading ? 'Loading…' : `${total} generations`}
            filters={(
              <Select value={status} onValueChange={(value) => value && setStatus(value)}>
                <SelectTrigger className="w-40"><SelectValue /></SelectTrigger>
                <SelectContent>
                  {statusOptions.map((option) => <SelectItem key={option} value={option}>{option === 'all' ? 'All statuses' : option}</SelectItem>)}
                </SelectContent>
              </Select>
            )}
          />
          {errorMessage ? <div className="mb-4 border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">{errorMessage}</div> : null}
          {loading ? <div className="py-8 text-sm text-slate-500">Loading image generations...</div> : (
            <>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Prompt</TableHead>
                    <TableHead>User</TableHead>
                    <TableHead>Status</TableHead>
                    <TableHead>Images</TableHead>
                    <TableHead>Created</TableHead>
                    <TableHead>Actions</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {items.length ? items.map((item) => (
                    <TableRow key={item.id}>
                      <TableCell>
                        <div className="min-w-0">
                          <div className="truncate font-medium">{item.prompt}</div>
                          <div className="text-xs text-slate-400">{item.model_slug}</div>
                        </div>
                      </TableCell>
                      <TableCell>
                        <div className="min-w-0">
                          <div className="truncate">{item.user_name || item.user_email}</div>
                          <div className="truncate text-xs text-slate-400">{item.user_email}</div>
                        </div>
                      </TableCell>
                      <TableCell><Badge variant={item.status === 'failed' ? 'destructive' : 'outline'}>{item.status}</Badge></TableCell>
                      <TableCell>{item.image_count}</TableCell>
                      <TableCell className="text-sm text-slate-500">{item.created_at ? new Date(item.created_at).toLocaleString() : '-'}</TableCell>
                      <TableCell>
                        <div className="flex gap-1">
                          <Button variant="ghost" size="sm" onClick={() => void handleView(item.id)}><Eye className="h-4 w-4" /></Button>
                          <Button variant="ghost" size="sm" onClick={() => setPendingDeleteId(item.id)}><Trash2 className="h-4 w-4" /></Button>
                        </div>
                      </TableCell>
                    </TableRow>
                  )) : (
                    <TableRow>
                      <TableCell colSpan={6} className="py-8 text-center text-sm text-slate-500">
                        No image generations match the current filters.
                      </TableCell>
                    </TableRow>
                  )}
                </TableBody>
              </Table>
              <AdminTablePagination
                page={page}
                pageSize={pageSize}
                visibleCount={items.length}
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

      <Dialog open={!!selected} onOpenChange={() => setSelected(null)}>
        <DialogContent className="max-w-4xl">
          <DialogHeader>
            <DialogTitle>Image generation detail</DialogTitle>
            <DialogDescription>{selected?.user_email}</DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div className="border p-4">
              <div className="text-xs uppercase tracking-[0.14em] text-slate-500">Prompt</div>
              <p className="mt-2 whitespace-pre-wrap text-sm text-slate-800">{selected?.prompt}</p>
            </div>
            {selected?.error_message ? <div className="border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">{selected.error_message}</div> : null}
            <div className="grid gap-3 md:grid-cols-2">
              {(selected?.result_urls || []).map((image, index) => (
                <div key={image.id || `${index}`} className="border p-3">
                  <div className="mb-2 text-xs uppercase tracking-[0.14em] text-slate-500">{image.id || `asset-${index + 1}`}</div>
                  {image.url ? <a href={image.url} target="_blank" rel="noreferrer" className="break-all text-sm text-blue-600 underline">{image.url}</a> : <span className="text-sm text-slate-400">No URL</span>}
                </div>
              ))}
            </div>
          </div>
        </DialogContent>
      </Dialog>

      <ConfirmActionDialog
        open={pendingDeleteId !== null}
        onOpenChange={(open) => {
          if (!open && !deletingId) setPendingDeleteId(null);
        }}
        title="Delete image generation?"
        description="Bản ghi generation này sẽ bị xoá khỏi admin images và không thể khôi phục từ giao diện."
        confirmLabel="Delete"
        loading={deletingId !== null}
        onConfirm={handleDelete}
      />
    </div>
  );
}
