"use client";

import { useEffect, useState } from "react";
import {
  CircleUserRound,
  LogOut,
  Settings,
} from "lucide-react";
import { createPortal } from "react-dom";
import { DevUserSwitcher } from "@/components/dev-user-switcher";
import { cn } from "@/lib/utils";

interface ChatConversationHeaderProps {
  title?: string;
}

const OPEN_CHAT_SETTINGS_EVENT = "chat:open-settings";

export function ChatConversationHeader({}: ChatConversationHeaderProps) {
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [mounted, setMounted] = useState(false);
  const canShowDevSwitcher = process.env.NODE_ENV !== "production";

  useEffect(() => {
    setMounted(true);

    const handleOpenSettings = () => setSettingsOpen(true);
    window.addEventListener(OPEN_CHAT_SETTINGS_EVENT, handleOpenSettings);

    return () => {
      window.removeEventListener(OPEN_CHAT_SETTINGS_EVENT, handleOpenSettings);
    };
  }, []);

  const handleLogout = async () => {
    await fetch("/api/auth/logout", { method: "POST", credentials: "include" });
    window.location.href = "/login";
  };

  return (
    <div className="fixed top-0 z-50 flex w-full items-center justify-between bg-[#131313]/60 px-6 py-4 backdrop-blur-2xl">
      <div className="md:hidden">
        <h1 className="font-[var(--font-chat-headline)] text-xl font-black tracking-tight text-white">
          CURATOR
        </h1>
      </div>

      <div className="relative flex items-center gap-4">
        <button
          type="button"
          onClick={() => setSettingsOpen((value) => !value)}
          className={cn(
            "rounded-full p-2 transition-colors duration-200",
            settingsOpen
              ? "bg-[#393939] text-white"
              : "text-white/60 hover:bg-[#393939]"
          )}
          aria-label="Settings"
        >
          <Settings className="h-5 w-5" />
        </button>
        <button
          type="button"
          onClick={() => setSettingsOpen(true)}
          className="rounded-full p-2 text-white/90 transition-colors duration-200 active:scale-95"
          aria-label="Account"
        >
          <CircleUserRound className="h-6 w-6" />
        </button>
      </div>

      {mounted && settingsOpen
        ? createPortal(
            <div className="fixed inset-0 z-[100]">
              <button
                type="button"
                aria-label="Close settings"
                className="absolute inset-0 bg-black/35 backdrop-blur-[2px]"
                onClick={() => setSettingsOpen(false)}
              />

              <div className="absolute left-1/2 top-1/2 w-[min(32rem,calc(100vw-2rem))] -translate-x-1/2 -translate-y-1/2 rounded-2xl border border-[#474747]/40 bg-[#1f1f1f] p-3 shadow-[0_24px_80px_rgba(0,0,0,0.55)]">
                <div className="border-b border-[#474747]/30 px-3 pb-3">
                  <p className="text-[10px] font-bold uppercase tracking-[0.16em] text-[#919191]">
                    Settings
                  </p>
                  <p className="mt-2 text-sm text-white">Workspace controls</p>
                  <p className="mt-1 text-xs leading-5 text-[#c6c6c6]">
                    Switch dev user, inspect the current session, or sign out.
                  </p>
                </div>

                <div className="mt-3 rounded-xl border border-[#474747]/40 bg-[#131313] px-4 py-3">
                  <div className="flex items-center gap-3">
                    <div className="flex h-10 w-10 items-center justify-center rounded-full bg-[#2a2a2a] text-white">
                      <CircleUserRound className="h-5 w-5" />
                    </div>
                    <div className="min-w-0">
                      <p className="text-sm font-medium text-white">Account</p>
                      <p className="text-xs text-[#919191]">
                        Session controls and dev switching
                      </p>
                    </div>
                  </div>
                </div>

                {canShowDevSwitcher ? (
                  <div className="pt-3">
                    <DevUserSwitcher embedded />
                  </div>
                ) : null}

                <button
                  type="button"
                  onClick={() => void handleLogout()}
                  className={cn(
                    "mt-3 flex w-full items-center gap-3 rounded-xl px-4 py-3 text-left text-sm transition-colors",
                    "text-white/70 hover:bg-[#2a2a2a] hover:text-white"
                  )}
                >
                  <LogOut className="h-4 w-4" />
                  <span>Logout</span>
                </button>
              </div>
            </div>,
            document.body
          )
        : null}
    </div>
  );
}
