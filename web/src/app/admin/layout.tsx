'use client';

import { useState, useSyncExternalStore } from 'react';
import { Inter } from 'next/font/google';
import { AdminSidebar } from '@/components/admin-sidebar';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { getAdminToken, setAdminToken, subscribeToAdminToken } from '@/lib/api';

const inter = Inter({ subsets: ['latin'] });

export const dynamic = 'force-dynamic';

export default function AdminLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const [draftToken, setDraftToken] = useState('');
  const adminToken = useSyncExternalStore(
    subscribeToAdminToken,
    getAdminToken,
    () => null,
  );
  const hasAdminToken = Boolean(adminToken);

  const handleSaveToken = () => {
    const normalizedToken = draftToken.trim();
    if (!normalizedToken) return;
    setAdminToken(normalizedToken);
    setDraftToken('');
  };

  if (!hasAdminToken) {
    return (
      <div className={`admin-shell min-h-screen bg-background text-foreground ${inter.className}`}>
        <div className="mx-auto flex min-h-screen max-w-5xl items-center px-6 py-12">
          <div className="grid w-full gap-8 lg:grid-cols-[minmax(0,1.1fr)_24rem]">
            <div className="space-y-4">
              <div className="text-xs font-semibold uppercase tracking-[0.18em] text-blue-600">Admin Access</div>
              <h1 className="max-w-lg text-4xl font-semibold leading-tight text-slate-950">
                Simple admin workspace for operations, billing, and account control.
              </h1>
              <p className="max-w-xl text-base text-slate-600">
                Dán `admin_token` từ backend để mở khóa toàn bộ admin. Giao diện này dùng tone sáng,
                ít bo góc, tập trung vào dữ liệu và thao tác nhanh.
              </p>
            </div>

            <Card className="w-full border-slate-200">
              <CardHeader>
                <CardTitle>Admin token</CardTitle>
                <CardDescription>
                  Token chỉ lưu local trong trình duyệt hiện tại.
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <Input
                  value={draftToken}
                  onChange={(event) => setDraftToken(event.target.value)}
                  placeholder="Paste admin token"
                />
                <Button onClick={handleSaveToken} className="w-full" disabled={!draftToken.trim()}>
                  Save Admin Token
                </Button>
              </CardContent>
            </Card>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={`admin-shell min-h-screen bg-background text-foreground ${inter.className}`}>
      <div className="flex min-h-screen">
        <AdminSidebar />
        <div className="flex min-h-screen flex-1 flex-col">
          <main className="flex-1 px-4 py-4 lg:px-6 lg:py-5">
            <div className="mx-auto max-w-7xl">{children}</div>
          </main>
        </div>
      </div>
    </div>
  );
}
