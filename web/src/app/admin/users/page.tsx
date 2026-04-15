'use client';

import { useEffect, useRef, useState } from 'react';
import { CreditCard, Eye, User } from 'lucide-react';
import { AdminTablePagination } from '@/components/admin/admin-table-pagination';
import { AdminTableToolbar } from '@/components/admin/admin-table-toolbar';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { adminFetch, type AdminListResponse } from '@/lib/api';
import { useDebouncedValue } from '@/lib/use-debounced-value';

interface UserItem {
  id: number;
  email: string;
  name: string | null;
  avatar_url: string | null;
  plan_name: string | null;
  balance: string;
  active: boolean;
  created_at: string;
}

interface UserDetail {
  id: number;
  email: string;
  name: string | null;
  avatar_url: string | null;
  provider: string;
  locale: string;
  active: boolean;
  plan: { id: number; name: string; slug: string } | null;
  balance: string;
  total_requests: number;
  transactions: Array<{
    id: number;
    tx_type: string;
    amount: string;
    currency: string;
    status: string;
    created_at: string;
  }>;
  created_at: string;
}

const pageSizeOptions = [10, 20, 50, 100];

export default function UsersPage() {
  const [users, setUsers] = useState<UserItem[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(true);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [search, setSearch] = useState('');
  const [planFilter, setPlanFilter] = useState('all');
  const [statusFilter, setStatusFilter] = useState('all');
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const [selectedUser, setSelectedUser] = useState<UserDetail | null>(null);
  const debouncedSearch = useDebouncedValue(search, 250);
  const requestIdRef = useRef(0);

  useEffect(() => {
    setPage(1);
  }, [debouncedSearch, planFilter, statusFilter]);

  useEffect(() => {
    const requestId = ++requestIdRef.current;

    const loadUsers = async () => {
      try {
        setLoading(true);
        setErrorMessage(null);
        const params = new URLSearchParams({
          page: String(page),
          limit: String(pageSize),
        });

        if (debouncedSearch) params.set('search', debouncedSearch);
        if (planFilter !== 'all') params.set('plan', planFilter);
        if (statusFilter !== 'all') params.set('active', String(statusFilter === 'active'));

        const data = await adminFetch<AdminListResponse<UserItem>>(`/admin/users?${params.toString()}`);
        if (requestId !== requestIdRef.current) return;
        setUsers(data.items);
        setTotal(data.total);
      } catch (error) {
        if (requestId !== requestIdRef.current) return;
        setErrorMessage(error instanceof Error ? error.message : 'Failed to load users.');
      } finally {
        if (requestId !== requestIdRef.current) return;
        setLoading(false);
      }
    };

    void loadUsers();
  }, [debouncedSearch, page, pageSize, planFilter, statusFilter]);

  const viewUserDetail = async (id: number) => {
    try {
      setErrorMessage(null);
      const data = await adminFetch<UserDetail>(`/admin/users/${id}`);
      setSelectedUser(data);
    } catch (error) {
      setErrorMessage(error instanceof Error ? error.message : 'Failed to load user detail.');
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Users</h1>
        <p className="mt-1 text-sm text-slate-500">Search users, filter by plan or status, and page through the full user list.</p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>User List</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <AdminTableToolbar
            searchValue={search}
            onSearchChange={setSearch}
            searchPlaceholder="Search users by name or email"
            summary={loading ? 'Loading…' : `${total} users found`}
            filters={(
              <>
                <Select value={planFilter} onValueChange={(value) => setPlanFilter(value || 'all')}>
                  <SelectTrigger className="w-40">
                    <SelectValue placeholder="Filter by plan" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">All Plans</SelectItem>
                    <SelectItem value="free">Free</SelectItem>
                    <SelectItem value="pro">Pro</SelectItem>
                    <SelectItem value="enterprise">Enterprise</SelectItem>
                  </SelectContent>
                </Select>
                <Select value={statusFilter} onValueChange={(value) => setStatusFilter(value || 'all')}>
                  <SelectTrigger className="w-40">
                    <SelectValue placeholder="Filter by status" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">All Statuses</SelectItem>
                    <SelectItem value="active">Active</SelectItem>
                    <SelectItem value="inactive">Inactive</SelectItem>
                  </SelectContent>
                </Select>
              </>
            )}
          />

          {errorMessage ? (
            <div className="border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
              {errorMessage}
            </div>
          ) : null}

          {loading ? (
            <div className="py-8 text-center text-slate-400">Loading...</div>
          ) : (
            <>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>User</TableHead>
                    <TableHead>Plan</TableHead>
                    <TableHead>Balance</TableHead>
                    <TableHead>Status</TableHead>
                    <TableHead>Registered</TableHead>
                    <TableHead>Actions</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {users.length ? users.map((user) => (
                    <TableRow key={user.id}>
                      <TableCell>
                        <div className="flex items-center gap-2">
                          <User className="h-4 w-4 text-slate-400" />
                          <div>
                            <div className="font-medium">{user.name || user.email}</div>
                            <div className="text-sm text-slate-400">{user.email}</div>
                          </div>
                        </div>
                      </TableCell>
                      <TableCell>
                        <Badge variant="secondary">{user.plan_name || 'None'}</Badge>
                      </TableCell>
                      <TableCell>
                        <div className="flex items-center gap-1">
                          <CreditCard className="h-3 w-3 text-slate-400" />
                          {user.balance}
                        </div>
                      </TableCell>
                      <TableCell>
                        <Badge variant={user.active ? 'default' : 'destructive'}>
                          {user.active ? 'Active' : 'Inactive'}
                        </Badge>
                      </TableCell>
                      <TableCell className="text-sm text-slate-400">
                        {new Date(user.created_at).toLocaleDateString()}
                      </TableCell>
                      <TableCell>
                        <Button variant="ghost" size="sm" onClick={() => viewUserDetail(user.id)}>
                          <Eye className="h-4 w-4" />
                        </Button>
                      </TableCell>
                    </TableRow>
                  )) : (
                    <TableRow>
                      <TableCell colSpan={6} className="py-8 text-center text-sm text-slate-500">
                        No users match the current filters.
                      </TableCell>
                    </TableRow>
                  )}
                </TableBody>
              </Table>

              <AdminTablePagination
                page={page}
                pageSize={pageSize}
                visibleCount={users.length}
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

      <Dialog open={!!selectedUser} onOpenChange={() => setSelectedUser(null)}>
        <DialogContent className="max-h-[80vh] max-w-2xl overflow-y-auto">
          <DialogHeader>
            <DialogTitle>User Details</DialogTitle>
            <DialogDescription>{selectedUser?.email}</DialogDescription>
          </DialogHeader>

          {selectedUser ? (
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="text-sm text-slate-400">Name</label>
                  <p className="font-medium">{selectedUser.name || 'N/A'}</p>
                </div>
                <div>
                  <label className="text-sm text-slate-400">Provider</label>
                  <p className="font-medium">{selectedUser.provider}</p>
                </div>
                <div>
                  <label className="text-sm text-slate-400">Plan</label>
                  <p className="font-medium">{selectedUser.plan?.name || 'None'}</p>
                </div>
                <div>
                  <label className="text-sm text-slate-400">Balance</label>
                  <p className="font-medium">{selectedUser.balance}</p>
                </div>
                <div>
                  <label className="text-sm text-slate-400">Total Requests</label>
                  <p className="font-medium">{selectedUser.total_requests}</p>
                </div>
                <div>
                  <label className="text-sm text-slate-400">Status</label>
                  <Badge variant={selectedUser.active ? 'default' : 'destructive'}>
                    {selectedUser.active ? 'Active' : 'Inactive'}
                  </Badge>
                </div>
              </div>

              <div>
                <h3 className="mb-2 font-medium">Recent Transactions</h3>
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Type</TableHead>
                      <TableHead>Amount</TableHead>
                      <TableHead>Status</TableHead>
                      <TableHead>Date</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {selectedUser.transactions.map((transaction) => (
                      <TableRow key={transaction.id}>
                        <TableCell>{transaction.tx_type}</TableCell>
                        <TableCell>{transaction.amount} {transaction.currency}</TableCell>
                        <TableCell>
                          <Badge variant={transaction.status === 'completed' ? 'default' : 'secondary'}>
                            {transaction.status}
                          </Badge>
                        </TableCell>
                        <TableCell>{new Date(transaction.created_at).toLocaleString()}</TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </div>
            </div>
          ) : null}
        </DialogContent>
      </Dialog>
    </div>
  );
}
