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
  deleteAdminConversation,
  getAdminConversationDetail,
  listAdminConversations,
  type AdminConversationDetail,
  type AdminConversationListItem,
} from '@/lib/api';

const modelOptions = ['all', 'grok-3', 'grok-4'];
const pageSizeOptions = [10, 20, 50, 100];

export default function AdminConversationsPage() {
  const [items, setItems] = useState<AdminConversationListItem[]>([]);
  const [total, setTotal] = useState(0);
  const [selected, setSelected] = useState<AdminConversationDetail | null>(null);
  const [pendingDeleteId, setPendingDeleteId] = useState<number | null>(null);
  const [deletingId, setDeletingId] = useState<number | null>(null);
  const [search, setSearch] = useState('');
  const [model, setModel] = useState('all');
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const requestIdRef = useRef(0);

  useEffect(() => {
    setPage(1);
  }, [search, model]);

  useEffect(() => {
    const requestId = ++requestIdRef.current;

    async function load() {
      try {
        setLoading(true);
        setErrorMessage(null);
        const data = await listAdminConversations(search, model, pageSize, (page - 1) * pageSize);
        if (requestId !== requestIdRef.current) return;
        setItems(data.items);
        setTotal(data.total);
      } catch (error) {
        if (requestId !== requestIdRef.current) return;
        setErrorMessage(error instanceof Error ? error.message : 'Failed to load conversations.');
      } finally {
        if (requestId !== requestIdRef.current) return;
        setLoading(false);
      }
    }

    void load();
  }, [model, page, pageSize, search]);

  async function handleView(id: number) {
    try {
      setSelected(await getAdminConversationDetail(id));
    } catch (error) {
      setErrorMessage(error instanceof Error ? error.message : 'Failed to load conversation detail.');
    }
  }

  async function handleDelete() {
    if (pendingDeleteId === null) return;

    try {
      setDeletingId(pendingDeleteId);
      setErrorMessage(null);
      await deleteAdminConversation(pendingDeleteId);
      setItems((current) => current.filter((item) => item.id !== pendingDeleteId));
      if (selected?.id === pendingDeleteId) setSelected(null);
      setPendingDeleteId(null);
    } catch (error) {
      setErrorMessage(error instanceof Error ? error.message : 'Failed to delete conversation.');
    } finally {
      setDeletingId(null);
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
        <div>
          <h1 className="text-3xl font-bold">Conversations</h1>
          <p className="mt-1 text-sm text-slate-500">Giám sát chat threads của user, đọc nội dung, và xoá khi cần.</p>
        </div>
      </div>

      <Card>
        <CardHeader><CardTitle>Conversation list</CardTitle></CardHeader>
        <CardContent>
          <AdminTableToolbar
            searchValue={search}
            onSearchChange={setSearch}
            searchPlaceholder="Search user or title"
            summary={loading ? 'Loading…' : `${total} conversations`}
            filters={(
              <Select value={model} onValueChange={(value) => value && setModel(value)}>
                <SelectTrigger className="w-40"><SelectValue /></SelectTrigger>
                <SelectContent>
                  {modelOptions.map((option) => <SelectItem key={option} value={option}>{option === 'all' ? 'All models' : option}</SelectItem>)}
                </SelectContent>
              </Select>
            )}
          />
          {errorMessage ? <div className="mb-4 border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">{errorMessage}</div> : null}
          {loading ? <div className="py-8 text-sm text-slate-500">Loading conversations...</div> : (
            <>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Conversation</TableHead>
                    <TableHead>User</TableHead>
                    <TableHead>Model</TableHead>
                    <TableHead>Messages</TableHead>
                    <TableHead>Updated</TableHead>
                    <TableHead>Actions</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {items.length ? items.map((item) => (
                    <TableRow key={item.id}>
                      <TableCell>
                        <div className="min-w-0">
                          <div className="truncate font-medium">{item.title || 'Untitled conversation'}</div>
                          <div className="text-xs text-slate-400">#{item.id}</div>
                        </div>
                      </TableCell>
                      <TableCell>
                        <div className="min-w-0">
                          <div className="truncate">{item.user_name || item.user_email}</div>
                          <div className="truncate text-xs text-slate-400">{item.user_email}</div>
                        </div>
                      </TableCell>
                      <TableCell><Badge variant="outline">{item.model_slug || 'n/a'}</Badge></TableCell>
                      <TableCell>{item.message_count}</TableCell>
                      <TableCell className="text-sm text-slate-500">{item.updated_at ? new Date(item.updated_at).toLocaleString() : '-'}</TableCell>
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
                        No conversations match the current filters.
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
            <DialogTitle>{selected?.title || 'Conversation detail'}</DialogTitle>
            <DialogDescription>{selected?.user_email}</DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div className="flex flex-wrap gap-2">
              <Badge variant="outline">{selected?.model_slug || 'n/a'}</Badge>
              <Badge variant="outline">{selected?.messages.length || 0} messages</Badge>
            </div>
            <div className="max-h-[60vh] space-y-3 overflow-y-auto border p-4">
              {selected?.messages.map((message) => (
                <div key={message.id} className="border p-3">
                  <div className="mb-2 flex items-center justify-between text-xs uppercase tracking-[0.14em] text-slate-500">
                    <span>{message.role}</span>
                    <span>{message.created_at ? new Date(message.created_at).toLocaleString() : '-'}</span>
                  </div>
                  <p className="whitespace-pre-wrap text-sm text-slate-800">{message.content}</p>
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
        title="Delete conversation?"
        description="Cuộc hội thoại sẽ bị ẩn khỏi admin conversations và không còn xuất hiện trong list đang active."
        confirmLabel="Delete"
        loading={deletingId !== null}
        onConfirm={handleDelete}
      />
    </div>
  );
}
