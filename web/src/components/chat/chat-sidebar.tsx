"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import {
  Archive,
  Clapperboard,
  HelpCircle,
  History,
  Inbox,
  ImageIcon,
  MessageSquare,
  MoreVertical,
  PlusCircle,
  Settings,
} from "lucide-react";
import { ConfirmActionDialog } from "@/components/confirm-action-dialog";
import { Button } from "@/components/ui/button";
import {
  deleteConversation,
  hasActiveUserSession,
  listConversations,
  type ConversationListItem,
} from "@/lib/user-api";
import { cn } from "@/lib/utils";

const CHAT_CONVERSATIONS_CHANGED_EVENT = "chat:conversations-changed";
const OPEN_CHAT_SETTINGS_EVENT = "chat:open-settings";

export function ChatSidebar({ onNavigate }: { onNavigate?: () => void }) {
  const pathname = usePathname();
  const router = useRouter();
  const [items, setItems] = useState<ConversationListItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [hasSession, setHasSession] = useState<boolean | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [pendingDelete, setPendingDelete] = useState<ConversationListItem | null>(null);
  const [openActionMenuId, setOpenActionMenuId] = useState<number | null>(null);
  const [deleting, setDeleting] = useState(false);

  useEffect(() => {
    let alive = true;

    const load = async () => {
      try {
        setError(null);
        const authenticated = await hasActiveUserSession();
        if (!alive) return;

        setHasSession(authenticated);
        if (!authenticated) {
          setItems([]);
          return;
        }

        const data = await listConversations();
        if (alive) setItems(data);
      } catch (nextError) {
        if (alive) {
          setError(nextError instanceof Error ? nextError.message : "Failed to load conversations");
        }
      } finally {
        if (alive) setLoading(false);
      }
    };

    void load();
    const handleRefresh = () => void load();
    window.addEventListener(CHAT_CONVERSATIONS_CHANGED_EVENT, handleRefresh);

    return () => {
      alive = false;
      window.removeEventListener(CHAT_CONVERSATIONS_CHANGED_EVENT, handleRefresh);
    };
  }, [pathname]);

  useEffect(() => {
    const handlePointerDown = () => {
      setOpenActionMenuId(null);
    };

    window.addEventListener("pointerdown", handlePointerDown);
    return () => {
      window.removeEventListener("pointerdown", handlePointerDown);
    };
  }, []);

  const handleDelete = async () => {
    if (!pendingDelete) return;
    try {
      setDeleting(true);
      await deleteConversation(pendingDelete.id);
      window.dispatchEvent(new Event(CHAT_CONVERSATIONS_CHANGED_EVENT));
      if (pathname === `/chat/${pendingDelete.id}`) {
        router.push("/chat");
      }
      setPendingDelete(null);
      setOpenActionMenuId(null);
    } finally {
      setDeleting(false);
    }
  };

  return (
    <>
      <aside className="hidden h-screen w-72 flex-col gap-2 border-r border-white/5 bg-[#131313] p-4 md:flex">
        <div className="mb-8 px-2">
          <h1 className="font-[var(--font-chat-headline)] text-lg font-black tracking-tight text-white">
            CURATOR
          </h1>
        </div>

        <div className="flex flex-1 flex-col gap-6 overflow-y-auto">
          <div>
            <p className="mb-4 px-2 text-[10px] font-bold uppercase tracking-[0.15em] text-[#c6c6c6]">
              Studio
            </p>
            <nav className="flex flex-col gap-1">
              <Link
                href="/chat"
                onClick={onNavigate}
                className={cn(
                  "flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm tracking-wide transition-all duration-200",
                  pathname.startsWith("/chat")
                    ? "bg-[#1f1f1f] text-white"
                    : "text-white/60 hover:bg-[#1f1f1f] hover:text-white"
                )}
              >
                <MessageSquare className="h-4 w-4" />
                <span>Chat</span>
              </Link>
              <Link
                href="/images"
                onClick={onNavigate}
                className={cn(
                  "flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm tracking-wide transition-all duration-200",
                  pathname.startsWith("/images")
                    ? "bg-[#1f1f1f] text-white"
                    : "text-white/60 hover:bg-[#1f1f1f] hover:text-white"
                )}
              >
                <ImageIcon className="h-4 w-4" />
                <span>Images</span>
              </Link>
              <Link
                href="/videos"
                onClick={onNavigate}
                className={cn(
                  "flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm tracking-wide transition-all duration-200",
                  pathname.startsWith("/videos")
                    ? "bg-[#1f1f1f] text-white"
                    : "text-white/60 hover:bg-[#1f1f1f] hover:text-white"
                )}
              >
                <Clapperboard className="h-4 w-4" />
                <span>Videos</span>
              </Link>
            </nav>
          </div>

          <div>
            <p className="mb-4 px-2 text-[10px] font-bold uppercase tracking-[0.15em] text-[#c6c6c6]">
              Conversations
            </p>
            <nav className="flex flex-col gap-1">
              <Link
                href="/chat?new=1"
                onClick={onNavigate}
                className="flex items-center gap-3 rounded-lg bg-[#393939] px-3 py-2.5 text-sm tracking-wide text-white transition-all duration-200"
              >
                <PlusCircle className="h-4 w-4" />
                <span>New Chat</span>
              </Link>

              {items.length ? (
                items.slice(0, 10).map((item) => {
                  const active = pathname === `/chat/${item.id}`;
                  const menuOpen = openActionMenuId === item.id;
                  return (
                    <div
                      key={item.id}
                      className={cn(
                        "group relative flex items-start gap-1 rounded-lg transition-all duration-200",
                        active
                          ? "bg-[#1f1f1f]"
                          : "hover:bg-[#1f1f1f]"
                      )}
                    >
                      <Link
                        href={`/chat/${item.id}`}
                        onClick={onNavigate}
                        className={cn(
                          "flex min-w-0 flex-1 items-center gap-3 px-3 py-2.5 text-sm tracking-wide transition-all duration-200",
                          active
                            ? "text-white"
                            : "text-white/60 group-hover:text-white"
                        )}
                      >
                        <History className="h-4 w-4 shrink-0" />
                        <div className="min-w-0">
                          <div className="truncate">{item.title || "Recent Idea"}</div>
                        </div>
                      </Link>
                      <Button
                        size="icon-sm"
                        variant="ghost"
                        className={cn(
                          "mt-1 text-[#919191] transition-opacity hover:bg-transparent hover:text-white",
                          menuOpen
                            ? "opacity-100"
                            : "opacity-0 group-hover:opacity-100 focus-visible:opacity-100"
                        )}
                        onPointerDown={(event) => {
                          event.stopPropagation();
                        }}
                        onClick={(event) => {
                          event.preventDefault();
                          event.stopPropagation();
                          setOpenActionMenuId((current) =>
                            current === item.id ? null : item.id
                          );
                        }}
                      >
                        <MoreVertical className="h-4 w-4" />
                      </Button>
                      {menuOpen ? (
                        <div
                          className="absolute right-1 top-11 z-20 min-w-36 overflow-hidden rounded-xl border border-white/10 bg-[#171717] p-1 shadow-[0_18px_48px_rgba(0,0,0,0.45)]"
                          onPointerDown={(event) => {
                            event.preventDefault();
                            event.stopPropagation();
                          }}
                          onClick={(event) => {
                            event.preventDefault();
                            event.stopPropagation();
                          }}
                        >
                          <button
                            type="button"
                            className="flex w-full items-center rounded-lg px-3 py-2 text-left text-sm text-rose-200 transition-colors hover:bg-white/6 hover:text-rose-100"
                            onPointerDown={(event) => {
                              event.preventDefault();
                              event.stopPropagation();
                            }}
                            onClick={() => {
                              setPendingDelete(item);
                              setOpenActionMenuId(null);
                            }}
                          >
                            Delete
                          </button>
                        </div>
                      ) : null}
                    </div>
                  );
                })
              ) : (
                <div className="rounded-lg px-3 py-2.5 text-sm tracking-wide text-white/60">
                  <div className="flex items-center gap-3">
                    <History className="h-4 w-4" />
                    <span>Recent Ideas</span>
                  </div>
                </div>
              )}

              <div className="flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm tracking-wide text-white/60 transition-all duration-200 hover:bg-[#1f1f1f] hover:text-white">
                <Inbox className="h-4 w-4" />
                <span>Archived</span>
              </div>
              <div className="flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm tracking-wide text-white/60 transition-all duration-200 hover:bg-[#1f1f1f] hover:text-white">
                <Archive className="h-4 w-4" />
                <span>Trash</span>
              </div>
            </nav>

            {loading ? <p className="px-3 pt-3 text-xs text-[#919191]">Loading conversations...</p> : null}
            {error ? <p className="px-3 pt-3 text-xs text-rose-300">{error}</p> : null}
          </div>

          <div className="mt-auto">
            <div className="mb-4 rounded-xl bg-[#1f1f1f] p-4">
              <p className="mb-1 text-sm font-bold text-white">Upgrade to Pro</p>
              <p className="mb-3 text-xs text-[#c6c6c6]">
                Unlock cinematic image generation and 4K video exports.
              </p>
              <button
                type="button"
                className="w-full rounded-lg bg-white py-2 text-xs font-bold uppercase tracking-wider text-[#1a1c1c] transition-colors hover:bg-[#c8c6c5]"
              >
                Learn More
              </button>
            </div>

            <nav className="flex flex-col gap-1">
              <button
                type="button"
                className="flex items-center gap-3 rounded-lg px-3 py-2 text-left text-sm tracking-wide text-white/60 transition-all duration-200 hover:bg-[#1f1f1f] hover:text-white"
              >
                <HelpCircle className="h-4 w-4" />
                <span>Help</span>
              </button>
              <button
                type="button"
                onClick={() => {
                  if (typeof window !== "undefined") {
                    window.dispatchEvent(new Event(OPEN_CHAT_SETTINGS_EVENT));
                  }
                }}
                className="flex items-center gap-3 rounded-lg px-3 py-2 text-left text-sm tracking-wide text-white/60 transition-all duration-200 hover:bg-[#1f1f1f] hover:text-white"
              >
                <Settings className="h-4 w-4" />
                <span>Settings</span>
              </button>
            </nav>
          </div>
        </div>
      </aside>

      <ConfirmActionDialog
        open={Boolean(pendingDelete)}
        onOpenChange={(open) => !open && setPendingDelete(null)}
        title="Delete conversation?"
        description="This removes the conversation from your workspace."
        confirmLabel="Delete"
        theme="chat"
        loading={deleting}
        onConfirm={handleDelete}
      />
    </>
  );
}

export function notifyChatSidebarChanged() {
  if (typeof window !== "undefined") {
    window.dispatchEvent(new Event(CHAT_CONVERSATIONS_CHANGED_EVENT));
  }
}
