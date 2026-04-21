'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import {
  Activity,
  Boxes,
  CreditCard,
  FileText,
  ImageIcon,
  Key,
  LayoutDashboard,
  LogOut,
  MessageSquare,
  Server,
  TrendingUp,
  Users,
} from 'lucide-react';
import { clearAdminToken } from '@/lib/api';

const navItems = [
  { href: '/admin', icon: LayoutDashboard, label: 'Dashboard' },
  { href: '/admin/users', icon: Users, label: 'Users' },
  { href: '/admin/conversations', icon: MessageSquare, label: 'Conversations' },
  { href: '/admin/images', icon: ImageIcon, label: 'Images' },
  { href: '/admin/payments', icon: CreditCard, label: 'Payments' },
  { href: '/admin/plans', icon: FileText, label: 'Plans' },
  { href: '/admin/providers', icon: Boxes, label: 'Providers' },
  { href: '/admin/revenue', icon: TrendingUp, label: 'Revenue' },
  { href: '/admin/usage', icon: Activity, label: 'Usage' },
  { href: '/admin/health', icon: Activity, label: 'Health' },
  { href: '/admin/accounts', icon: Server, label: 'Accounts' },
  { href: '/admin/proxies', icon: Server, label: 'Proxies' },
  { href: '/admin/api-keys', icon: Key, label: 'API Keys' },
];

export function AdminSidebar() {
  const pathname = usePathname();

  const handleLogout = () => {
    clearAdminToken();
    window.location.href = '/';
  };

  return (
    <aside className="sticky top-0 flex min-h-screen w-64 shrink-0 flex-col border-r bg-white">
      <div className="border-b px-5 py-5">
        <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-slate-500">Admin</div>
        <div className="mt-2 text-xl font-semibold text-slate-950">Operations</div>
        <div className="mt-1 flex items-center gap-2 text-sm text-slate-600">
          <span className="inline-flex h-2 w-2 rounded-full bg-green-500" />
          System online
        </div>
      </div>

      <nav className="flex-1 space-y-1 px-3 py-4">
        {navItems.map((item) => {
          const Icon = item.icon;
          const isActive =
            item.href === '/admin'
              ? pathname === '/admin'
              : pathname.startsWith(item.href);

          return (
            <Link
              key={item.href}
              href={item.href}
              className={`flex items-center gap-3 border px-3 py-2.5 text-sm font-medium transition-colors ${
                isActive
                  ? 'border-blue-200 bg-blue-50 text-blue-700'
                  : 'border-transparent text-slate-700 hover:border-slate-200 hover:bg-slate-50'
              }`}
            >
              <Icon className="h-4 w-4" />
              <span>{item.label}</span>
            </Link>
          );
        })}
      </nav>

      <div className="border-t p-3">
        <button
          onClick={handleLogout}
          className="flex w-full items-center gap-3 border border-transparent px-3 py-2.5 text-sm font-medium text-slate-600 transition-colors hover:border-red-200 hover:bg-red-50 hover:text-red-700"
        >
          <LogOut className="h-4 w-4" />
          <span>Logout</span>
        </button>
      </div>
    </aside>
  );
}
