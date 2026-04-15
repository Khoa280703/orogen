"use client";

import { useEffect, useState } from "react";
import { Check, Loader2, Search, Users } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

type DevUser = {
  id: number;
  email: string;
  name?: string | null;
};

type DevUsersResponse = {
  users: DevUser[];
  currentUserId: number | null;
};

interface DevUserSwitcherProps {
  embedded?: boolean;
}

function labelForUser(user: DevUser) {
  return user.name?.trim() ? `${user.name} (${user.email})` : user.email;
}

export function DevUserSwitcher({
  embedded = false,
}: DevUserSwitcherProps) {
  const [users, setUsers] = useState<DevUser[]>([]);
  const [query, setQuery] = useState("");
  const [activeUserId, setActiveUserId] = useState<string>("");
  const [selectedUserId, setSelectedUserId] = useState<string>("");
  const [loading, setLoading] = useState(true);
  const [switching, setSwitching] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const normalizedQuery = query.trim().toLowerCase();
  const filteredUsers = users.filter((user) => {
    if (!normalizedQuery) {
      return true;
    }

    return [user.name || "", user.email].some((value) =>
      value.toLowerCase().includes(normalizedQuery)
    );
  });

  useEffect(() => {
    if (process.env.NODE_ENV === "production") {
      return;
    }

    let active = true;

    async function load() {
      try {
        setLoading(true);
        setError(null);
        const response = await fetch("/api/dev/users", {
          credentials: "include",
          cache: "no-store",
        });

        if (!response.ok) {
          throw new Error("Failed to load users");
        }

        const data = (await response.json()) as DevUsersResponse;
        if (!active) {
          return;
        }

        setUsers(data.users);
        const defaultUserId =
          data.currentUserId
            ? String(data.currentUserId)
            : data.users[0]
              ? String(data.users[0].id)
              : "";

        setActiveUserId(data.currentUserId ? String(data.currentUserId) : "");
        setSelectedUserId(defaultUserId);
      } catch (nextError) {
        if (active) {
          setError(nextError instanceof Error ? nextError.message : "Failed to load users");
        }
      } finally {
        if (active) {
          setLoading(false);
        }
      }
    }

    void load();

    return () => {
      active = false;
    };
  }, []);

  if (process.env.NODE_ENV === "production") {
    return null;
  }

  const selectedUser = users.find((user) => String(user.id) === selectedUserId) ?? null;

  async function handleSwitch() {
    if (!selectedUserId || switching) {
      return;
    }

    try {
      setSwitching(true);
      setError(null);
      const response = await fetch("/api/dev/switch-user", {
        method: "POST",
        credentials: "include",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ userId: Number(selectedUserId) }),
      });

      if (!response.ok) {
        throw new Error("Failed to switch user");
      }

      window.location.replace("/chat?new=1");
    } catch (nextError) {
      setError(nextError instanceof Error ? nextError.message : "Failed to switch user");
      setSwitching(false);
    }
  }

  return (
    <div
      className={
        embedded
          ? "space-y-3 rounded-xl border border-[#474747]/40 bg-[#131313] p-4"
          : "space-y-3 rounded-3xl border border-amber-400/20 bg-amber-400/10 p-4"
      }
    >
      <div className={embedded ? "flex items-center gap-2 text-[#e2e2e2]" : "flex items-center gap-2 text-amber-100"}>
        <Users className="h-4 w-4" />
        <p className="text-sm font-medium">Dev user switcher</p>
      </div>

      <div className="relative">
        <Search
          className={
            embedded
              ? "pointer-events-none absolute top-1/2 left-2.5 h-3.5 w-3.5 -translate-y-1/2 text-[#919191]"
              : "pointer-events-none absolute top-1/2 left-2.5 h-3.5 w-3.5 -translate-y-1/2 text-amber-100/70"
          }
        />
        <Input
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder="Search name or email"
          className={
            embedded
              ? "border-[#474747]/40 bg-[#0e0e0e] pr-2 pl-8 text-white placeholder:text-[#919191]"
              : "border-white/10 bg-black/20 pr-2 pl-8 text-white placeholder:text-slate-400"
          }
        />
      </div>

      <div
        className={
          embedded
            ? "space-y-2 rounded-xl border border-[#474747]/40 bg-[#0e0e0e] p-2"
            : "space-y-2 rounded-2xl border border-white/10 bg-black/20 p-2"
        }
      >
        {loading ? (
          <p className={embedded ? "px-2 py-3 text-sm text-[#919191]" : "px-2 py-3 text-sm text-slate-400"}>
            Loading users...
          </p>
        ) : filteredUsers.length ? (
          <div className="max-h-64 space-y-2 overflow-y-auto pr-1">
            {filteredUsers.map((user) => {
              const userId = String(user.id);
              const isSelected = userId === selectedUserId;
              const isActive = userId === activeUserId;

              return (
                <button
                  key={user.id}
                  type="button"
                  onClick={() => setSelectedUserId(userId)}
                  className={
                    embedded
                      ? `flex w-full items-start justify-between rounded-xl border px-3 py-3 text-left transition-colors ${
                          isSelected
                            ? "border-white/40 bg-[#1f1f1f] text-white"
                            : "border-[#474747]/35 bg-[#151515] text-[#c6c6c6] hover:border-[#6b6b6b] hover:bg-[#1a1a1a]"
                        }`
                      : `flex w-full items-start justify-between rounded-2xl border px-3 py-3 text-left transition-colors ${
                          isSelected
                            ? "border-amber-300/60 bg-amber-300/10 text-white"
                            : "border-white/10 bg-black/20 text-slate-200 hover:border-white/20 hover:bg-black/30"
                        }`
                  }
                >
                  <div className="min-w-0 pr-3">
                    <div className="flex flex-wrap items-center gap-2">
                      <span className="truncate text-sm font-medium text-inherit">
                        {user.name?.trim() || "Unnamed user"}
                      </span>
                      {isActive ? (
                        <span className="rounded-full border border-emerald-400/30 bg-emerald-400/10 px-2 py-0.5 text-[10px] font-semibold uppercase tracking-[0.14em] text-emerald-300">
                          Current
                        </span>
                      ) : null}
                    </div>
                    <p className={embedded ? "mt-1 truncate text-xs text-[#919191]" : "mt-1 truncate text-xs text-slate-400"}>
                      {user.email}
                    </p>
                  </div>

                  <span
                    className={
                      embedded
                        ? `mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center rounded-full border ${
                            isSelected
                              ? "border-white bg-white text-[#111111]"
                              : "border-[#5a5a5a] text-transparent"
                          }`
                        : `mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center rounded-full border ${
                            isSelected
                              ? "border-amber-300 bg-amber-300 text-slate-950"
                              : "border-slate-500 text-transparent"
                          }`
                    }
                  >
                    <Check className="h-3.5 w-3.5" />
                  </span>
                </button>
              );
            })}
          </div>
        ) : (
          <p className={embedded ? "px-2 py-3 text-sm text-[#919191]" : "px-2 py-3 text-sm text-slate-400"}>
            No matching users found.
          </p>
        )}
      </div>

      {!loading ? (
        <p className={embedded ? "text-xs text-[#919191]" : "text-xs text-amber-100/75"}>
          {selectedUser ? `Selected: ${labelForUser(selectedUser)}` : `${filteredUsers.length} / ${users.length} users`}
        </p>
      ) : null}

      <Button
        type="button"
        size="sm"
        disabled={
          loading ||
          switching ||
          !selectedUserId ||
          !filteredUsers.some((user) => String(user.id) === selectedUserId)
        }
        onClick={() => void handleSwitch()}
        className={
          embedded
            ? "w-full bg-white text-[#1a1c1c] hover:bg-[#c8c6c5]"
            : "w-full bg-amber-300 text-slate-950 hover:bg-amber-200"
        }
      >
        {switching ? <Loader2 className="h-4 w-4 animate-spin" /> : null}
        {selectedUserId === activeUserId ? "Refresh current user" : "Switch user"}
      </Button>

      {error ? <p className="text-xs text-red-200">{error}</p> : null}
    </div>
  );
}
