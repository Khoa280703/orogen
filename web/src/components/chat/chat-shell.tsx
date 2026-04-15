"use client";

import { useState } from "react";
import { Menu, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ChatSidebar } from "@/components/chat/chat-sidebar";
import { cn } from "@/lib/utils";

export function ChatShell({
  children,
  className,
}: {
  children: React.ReactNode;
  className?: string;
}) {
  const [sidebarOpen, setSidebarOpen] = useState(false);

  return (
    <div className={cn("flex h-[100dvh] min-h-0 overflow-hidden bg-[#131313] text-[#e2e2e2]", className)}>
      <div className="hidden lg:block lg:w-72 lg:shrink-0 lg:overflow-hidden">
        <ChatSidebar />
      </div>

      {sidebarOpen ? (
        <div className="fixed inset-0 z-40 bg-black/70 p-3 backdrop-blur-sm lg:hidden">
          <div className="h-full w-80 max-w-[88vw] overflow-hidden rounded-[2rem] border border-white/10 bg-[#131314] shadow-[0_24px_80px_rgba(2,6,23,0.42)]">
            <ChatSidebar onNavigate={() => setSidebarOpen(false)} />
          </div>
        </div>
      ) : null}

      <div className="relative flex min-w-0 flex-1 flex-col overflow-hidden bg-[#131313]">
        <div className="pointer-events-none absolute left-[-5%] top-[-10%] h-[40%] w-[40%] rounded-full bg-white/5 blur-[120px]" />
        <div className="pointer-events-none absolute bottom-[-10%] right-[-5%] h-[30%] w-[30%] rounded-full bg-white/5 blur-[100px]" />
        <div className="flex items-center justify-between px-4 py-3 md:hidden">
          <Button
            size="icon-sm"
            variant="ghost"
            className="text-slate-200"
            onClick={() => setSidebarOpen((value) => !value)}
          >
            {sidebarOpen ? <X className="h-4 w-4" /> : <Menu className="h-4 w-4" />}
          </Button>
          <span className="font-[var(--font-chat-headline)] text-lg font-black tracking-tight text-white">
            CURATOR
          </span>
          <span className="w-7" />
        </div>
        <div className="flex min-h-0 flex-1 flex-col overflow-hidden">{children}</div>
      </div>
    </div>
  );
}
