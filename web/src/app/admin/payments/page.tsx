'use client';

import { useEffect, useMemo, useState } from 'react';
import { AdminTablePagination } from '@/components/admin/admin-table-pagination';
import { AdminTableToolbar } from '@/components/admin/admin-table-toolbar';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { adminFetch } from '@/lib/api';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Textarea } from '@/components/ui/textarea';
import { Check, X, DollarSign, User, Calendar } from 'lucide-react';

interface Payment {
  id: number;
  user_id: number;
  user_email: string | null;
  user_name: string | null;
  amount: string;
  currency: string;
  reference: string | null;
  proof_url: string | null;
  status: string;
  created_at: string;
}

export default function PaymentsPage() {
  const [payments, setPayments] = useState<Payment[]>([]);
  const [loading, setLoading] = useState(true);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [statusFilter, setStatusFilter] = useState('pending');
  const [approveDialog, setApproveDialog] = useState<{ open: boolean; payment: Payment | null; notes: string }>({
    open: false,
    payment: null,
    notes: '',
  });
  const [rejectDialog, setRejectDialog] = useState<{ open: boolean; payment: Payment | null; notes: string }>({
    open: false,
    payment: null,
    notes: '',
  });
  const [search, setSearch] = useState('');
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);

  useEffect(() => {
    loadPayments();
  }, [statusFilter]);

  useEffect(() => {
    setPage(1);
  }, [search, statusFilter]);

  const loadPayments = async () => {
    try {
      setErrorMessage(null);
      const data = await adminFetch<{ payments?: Payment[] }>('/admin/payments');
      setPayments(data.payments || []);
    } catch (error) {
      setErrorMessage(error instanceof Error ? error.message : 'Failed to load payments.');
    } finally {
      setLoading(false);
    }
  };

  const handleApprove = async () => {
    if (!approveDialog.payment) return;

    try {
      await adminFetch(`/admin/payments/${approveDialog.payment.id}/approve`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ notes: approveDialog.notes || undefined }),
      });
      setApproveDialog({ open: false, payment: null, notes: '' });
      setErrorMessage(null);
      loadPayments();
    } catch (error) {
      setErrorMessage(error instanceof Error ? error.message : 'Failed to approve payment.');
    }
  };

  const handleReject = async () => {
    if (!rejectDialog.payment) return;

    try {
      await adminFetch(`/admin/payments/${rejectDialog.payment.id}/reject`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ notes: rejectDialog.notes || undefined }),
      });
      setRejectDialog({ open: false, payment: null, notes: '' });
      setErrorMessage(null);
      loadPayments();
    } catch (error) {
      setErrorMessage(error instanceof Error ? error.message : 'Failed to reject payment.');
    }
  };

  const filteredPayments = useMemo(() => {
    const keyword = search.trim().toLowerCase();
    return payments.filter((payment) => {
      if (payment.status !== statusFilter) return false;
      if (!keyword) return true;
      return (
        (payment.user_name || '').toLowerCase().includes(keyword) ||
        (payment.user_email || '').toLowerCase().includes(keyword) ||
        (payment.reference || '').toLowerCase().includes(keyword) ||
        String(payment.id).includes(keyword)
      );
    });
  }, [payments, search, statusFilter]);

  const paginatedPayments = useMemo(() => {
    const start = (page - 1) * pageSize;
    return filteredPayments.slice(start, start + pageSize);
  }, [filteredPayments, page, pageSize]);

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h1 className="text-3xl font-bold">Payment Queue</h1>
      </div>

      {errorMessage && (
        <div className="border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
          {errorMessage}
        </div>
      )}

      <div className="grid gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Pending</CardTitle>
            <DollarSign className="h-4 w-4 text-slate-400" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-amber-500">
              {payments.filter(p => p.status === 'pending').length}
            </div>
            <p className="text-xs text-slate-400">Awaiting approval</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Completed</CardTitle>
            <DollarSign className="h-4 w-4 text-slate-400" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-green-500">
              {payments.filter(p => p.status === 'completed').length}
            </div>
            <p className="text-xs text-slate-400">Successfully processed</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Rejected</CardTitle>
            <DollarSign className="h-4 w-4 text-slate-400" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-red-500">
              {payments.filter(p => p.status === 'rejected').length}
            </div>
            <p className="text-xs text-slate-400">Declined transactions</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Transactions</CardTitle>
        </CardHeader>
        <CardContent>
          <AdminTableToolbar
            searchValue={search}
            onSearchChange={setSearch}
            searchPlaceholder="Search user, reference, or payment ID"
            summary={`${filteredPayments.length} payments`}
            filters={(
              <Select value={statusFilter} onValueChange={(value) => setStatusFilter(value || 'pending')}>
                <SelectTrigger className="w-40">
                  <SelectValue placeholder="Filter by status" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="pending">Pending</SelectItem>
                  <SelectItem value="completed">Completed</SelectItem>
                  <SelectItem value="rejected">Rejected</SelectItem>
                </SelectContent>
              </Select>
            )}
          />
          {loading ? (
            <div className="text-center py-8 text-slate-400">Loading...</div>
          ) : (
            <>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>ID</TableHead>
                    <TableHead>User</TableHead>
                    <TableHead>Amount</TableHead>
                    <TableHead>Reference</TableHead>
                    <TableHead>Proof</TableHead>
                    <TableHead>Status</TableHead>
                    <TableHead>Date</TableHead>
                    <TableHead>Actions</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {paginatedPayments.length ? paginatedPayments.map((payment) => (
                    <TableRow key={payment.id}>
                      <TableCell className="font-mono text-sm">#{payment.id}</TableCell>
                      <TableCell>
                        <div className="flex items-center gap-2">
                          <User className="h-4 w-4 text-slate-400" />
                          <div>
                            <div className="font-medium">{payment.user_name || payment.user_email || 'Unknown'}</div>
                            <div className="text-sm text-slate-400">{payment.user_email}</div>
                          </div>
                        </div>
                      </TableCell>
                      <TableCell className="font-medium">
                        {payment.amount} {payment.currency}
                      </TableCell>
                      <TableCell className="text-sm text-slate-400">
                        {payment.reference || '-'}
                      </TableCell>
                      <TableCell>
                        {payment.proof_url ? (
                          <a
                            href={payment.proof_url}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="text-blue-400 hover:underline text-sm"
                          >
                            View
                          </a>
                        ) : (
                          <span className="text-slate-500 text-sm">-</span>
                        )}
                      </TableCell>
                      <TableCell>
                        <Badge
                          variant={
                            payment.status === 'completed'
                              ? 'default'
                              : payment.status === 'rejected'
                                ? 'destructive'
                                : 'secondary'
                          }
                        >
                          {payment.status}
                        </Badge>
                      </TableCell>
                      <TableCell>
                        <div className="flex items-center gap-1 text-sm text-slate-400">
                          <Calendar className="h-3 w-3" />
                          {new Date(payment.created_at).toLocaleDateString()}
                        </div>
                      </TableCell>
                      <TableCell>
                        {payment.status === 'pending' && (
                          <div className="flex gap-2">
                            <Button
                              size="sm"
                              variant="default"
                              onClick={() => setApproveDialog({ open: true, payment, notes: '' })}
                            >
                              <Check className="h-4 w-4" />
                            </Button>
                            <Button
                              size="sm"
                              variant="destructive"
                              onClick={() => setRejectDialog({ open: true, payment, notes: '' })}
                            >
                              <X className="h-4 w-4" />
                            </Button>
                          </div>
                        )}
                      </TableCell>
                    </TableRow>
                  )) : (
                    <TableRow>
                      <TableCell colSpan={8} className="py-8 text-center text-sm text-slate-500">
                        No payments match the current filters.
                      </TableCell>
                    </TableRow>
                  )}
                </TableBody>
              </Table>
              <AdminTablePagination
                page={page}
                pageSize={pageSize}
                visibleCount={paginatedPayments.length}
                totalCount={filteredPayments.length}
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

      <Dialog open={approveDialog.open} onOpenChange={(open) => setApproveDialog({ ...approveDialog, open })}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Approve Payment</DialogTitle>
            <DialogDescription>
              Approve payment of {approveDialog.payment?.amount} {approveDialog.payment?.currency} for {approveDialog.payment?.user_email}
            </DialogDescription>
          </DialogHeader>
          <div className="py-4">
            <label className="text-sm text-slate-400">Notes (optional)</label>
            <Textarea
              value={approveDialog.notes}
              onChange={(e) => setApproveDialog({ ...approveDialog, notes: e.target.value })}
              placeholder="Add approval notes..."
              className="mt-2"
            />
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setApproveDialog({ open: false, payment: null, notes: '' })}>
              Cancel
            </Button>
            <Button onClick={handleApprove}>
              Approve Payment
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={rejectDialog.open} onOpenChange={(open) => setRejectDialog({ ...rejectDialog, open })}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Reject Payment</DialogTitle>
            <DialogDescription>
              Reject payment of {rejectDialog.payment?.amount} {rejectDialog.payment?.currency} for {rejectDialog.payment?.user_email}
            </DialogDescription>
          </DialogHeader>
          <div className="py-4">
            <label className="text-sm text-slate-400">Rejection Reason (optional)</label>
            <Textarea
              value={rejectDialog.notes}
              onChange={(e) => setRejectDialog({ ...rejectDialog, notes: e.target.value })}
              placeholder="Add rejection reason..."
              className="mt-2"
            />
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setRejectDialog({ open: false, payment: null, notes: '' })}>
              Cancel
            </Button>
            <Button variant="destructive" onClick={handleReject}>
              Reject Payment
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
