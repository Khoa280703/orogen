import { createHmac } from "node:crypto";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { getServerApiOrigin } from "@/lib/api-base-url";

type DevUserItem = {
  id: number;
  email: string;
  name?: string | null;
  avatar_url?: string | null;
  active?: boolean;
};

type DevUserListResponse = {
  items: DevUserItem[];
  total: number;
};

type RootConfig = {
  adminToken?: string;
};

function base64UrlEncode(value: string) {
  return Buffer.from(value)
    .toString("base64")
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=+$/g, "");
}

function base64UrlDecode(value: string) {
  const normalized = value.replace(/-/g, "+").replace(/_/g, "/");
  const padding = normalized.length % 4;
  const padded = padding ? normalized.padEnd(normalized.length + (4 - padding), "=") : normalized;
  return Buffer.from(padded, "base64").toString("utf8");
}

export function isDevAuthEnabled() {
  return process.env.NODE_ENV !== "production";
}

export async function readAdminToken() {
  if (process.env.ADMIN_TOKEN) {
    return process.env.ADMIN_TOKEN;
  }

  const configPath = path.resolve(process.cwd(), "../config.json");
  const raw = await readFile(configPath, "utf8");
  const config = JSON.parse(raw) as RootConfig;
  return config.adminToken || "";
}

export function signDevAccessToken(userId: number, email: string) {
  const jwtSecret =
    process.env.JWT_SECRET || "default-secret-change-in-production";
  const now = Math.floor(Date.now() / 1000);
  const payload = {
    user_id: userId,
    email,
    iat: now,
    exp: now + 60 * 60 * 24 * 30,
  };
  const header = { alg: "HS256", typ: "JWT" };
  const encodedHeader = base64UrlEncode(JSON.stringify(header));
  const encodedPayload = base64UrlEncode(JSON.stringify(payload));
  const unsignedToken = `${encodedHeader}.${encodedPayload}`;
  const signature = createHmac("sha256", jwtSecret)
    .update(unsignedToken)
    .digest("base64")
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=+$/g, "");

  return `${unsignedToken}.${signature}`;
}

export function readCurrentUserIdFromToken(token?: string | null) {
  if (!token) {
    return null;
  }

  try {
    const [, payload] = token.split(".");
    if (!payload) {
      return null;
    }

    const decoded = JSON.parse(base64UrlDecode(payload)) as { user_id?: number };
    return typeof decoded.user_id === "number" ? decoded.user_id : null;
  } catch {
    return null;
  }
}

export async function listDevUsers(): Promise<DevUserItem[]> {
  const adminToken = await readAdminToken();
  if (!adminToken) {
    throw new Error("Missing admin token");
  }

  const response = await fetch(`${getServerApiOrigin()}/admin/users?page=1&limit=100`, {
    headers: {
      Authorization: `Bearer ${adminToken}`,
    },
    cache: "no-store",
  });

  if (!response.ok) {
    throw new Error(`Failed to load users: ${response.status}`);
  }

  const data = (await response.json()) as DevUserListResponse;
  return data.items.filter((user) => user.active !== false);
}
