"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import {
  BarChart3,
  Clapperboard,
  CreditCard,
  ImageIcon,
  Key,
  LayoutDashboard,
  LogOut,
  MessageSquare,
  Settings,
} from "lucide-react";
import { DevUserSwitcher } from "@/components/dev-user-switcher";
import { Button } from "@/components/ui/button";

type NavItem = {
  href: string;
  label: string;
  icon: typeof MessageSquare;
  matchPrefix?: boolean;
};

const primaryItems: NavItem[] = [
  { href: "/chat", label: "Chat", icon: MessageSquare, matchPrefix: true },
  { href: "/images", label: "Images", icon: ImageIcon, matchPrefix: true },
  { href: "/videos", label: "Videos", icon: Clapperboard, matchPrefix: true },
];

const secondaryItems: NavItem[] = [
  { href: "/dashboard", label: "Overview", icon: LayoutDashboard },
  { href: "/dashboard/usage", label: "Usage", icon: BarChart3 },
  { href: "/dashboard/billing", label: "Billing", icon: CreditCard },
  { href: "/dashboard/keys", label: "API Keys", icon: Key },
  { href: "/dashboard/settings", label: "Settings", icon: Settings },
];

export function UserSidebar() {
  const pathname = usePathname();

  const handleLogout = async () => {
    await fetch("/api/auth/logout", { method: "POST", credentials: "include" });
    window.location.href = "/login";
  };

  const renderItems = (items: NavItem[]) =>
    items.map((item) => {
      const isActive = item.matchPrefix ? pathname.startsWith(item.href) : pathname === item.href;
      return (
        <Link key={item.href} href={item.href}>
          <div
            className={`flex items-center gap-3 rounded-[var(--radius)] px-4 py-3 transition-colors ${
              isActive ? "bg-blue-500/10 text-blue-300" : "text-slate-400 hover:bg-slate-800 hover:text-white"
            }`}
          >
            <item.icon className="h-5 w-5" />
            <span className="text-sm font-medium">{item.label}</span>
          </div>
        </Link>
      );
    });

  return (
    <div className="flex min-h-screen w-64 flex-col border-r border-slate-800 bg-slate-950 p-4">
      <div className="mb-8 px-4">
        <h1 className="text-xl font-bold bg-gradient-to-r from-emerald-300 via-blue-300 to-cyan-300 bg-clip-text text-transparent">
          Media Studio
        </h1>
      </div>

      <nav className="space-y-6">
        <div className="space-y-2">
          <p className="px-4 text-[11px] uppercase tracking-[0.18em] text-slate-500">Create</p>
          {renderItems(primaryItems)}
        </div>
        <div className="space-y-2 border-t border-slate-800 pt-6">
          <p className="px-4 text-[11px] uppercase tracking-[0.18em] text-slate-500">Manage</p>
          {renderItems(secondaryItems)}
        </div>
      </nav>

      <div className="mt-auto space-y-4 pt-6">
        <DevUserSwitcher />
        <Button
          onClick={handleLogout}
          variant="ghost"
          className="w-full justify-start text-slate-400 hover:bg-red-500/10 hover:text-red-400"
        >
          <LogOut className="mr-3 h-5 w-5" />
          Sign Out
        </Button>
      </div>
    </div>
  );
}
