"use client";

import { usePathname } from "next/navigation";
import { UserSidebar } from "@/components/user-sidebar";

export function AppLayoutShell({
  children,
  className,
}: {
  children: React.ReactNode;
  className: string;
}) {
  const pathname = usePathname();
  const isStudioRoute =
    pathname.startsWith("/chat") ||
    pathname.startsWith("/images") ||
    pathname.startsWith("/videos");

  if (isStudioRoute) {
    return <div className={`min-h-screen bg-slate-950 text-white ${className}`}>{children}</div>;
  }

  return (
    <div className={`min-h-screen bg-slate-950 text-white ${className}`}>
      <div className="flex">
        <UserSidebar />
        <main className="flex-1 p-8">{children}</main>
      </div>
    </div>
  );
}
