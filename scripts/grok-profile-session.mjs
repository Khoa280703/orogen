#!/usr/bin/env node

import { mkdir } from "node:fs/promises";
import { existsSync } from "node:fs";
import { spawn, spawnSync } from "node:child_process";
import { setTimeout as delay } from "node:timers/promises";

const [, , command, rawProfileDir] = process.argv;

if (!command || !rawProfileDir) {
  fail("Usage: node scripts/grok-profile-session.mjs <launch-login|sync-cookies> <profile-dir>");
}

const profileDir = rawProfileDir.trim();
if (!profileDir) {
  fail("Profile directory cannot be empty.");
}

const chromeBin = resolveChromeBinary();
if (!chromeBin) {
  fail("Cannot find Chrome/Chromium binary. Set GOOGLE_CHROME_BIN or install google-chrome.");
}

await mkdir(profileDir, { recursive: true });

if (command === "launch-login") {
  const child = spawn(
    chromeBin,
    [
      `--user-data-dir=${profileDir}`,
      "--no-first-run",
      "--no-default-browser-check",
      "--new-window",
      "https://grok.com/",
    ],
    {
      detached: true,
      stdio: "ignore",
      env: buildChromeEnv({ requireDisplay: true }),
    },
  );

  child.unref();
  emit({
    ok: true,
    profileDir,
    pid: child.pid,
    message: "Browser launched. Finish login in that window, then close it before syncing cookies.",
  });
  process.exit(0);
}

if (command !== "sync-cookies") {
  fail(`Unknown command: ${command}`);
}

const port = 9222 + Math.floor(Math.random() * 1000);
const chrome = spawn(
  chromeBin,
  [
    `--user-data-dir=${profileDir}`,
    "--headless=new",
    "--disable-gpu",
    "--no-first-run",
    "--no-default-browser-check",
    "--remote-debugging-address=127.0.0.1",
    `--remote-debugging-port=${port}`,
    "about:blank",
  ],
  {
    stdio: ["ignore", "ignore", "pipe"],
    env: buildChromeEnv({ requireDisplay: false }),
  },
);

let stderr = "";
chrome.stderr?.on("data", (chunk) => {
  stderr += chunk.toString();
});

try {
  const pageWsUrl = await waitForDebuggerTarget(port);
  const cdp = await connectCdp(pageWsUrl);

  try {
    await cdp.send("Page.enable");
    await cdp.send("Network.enable");
    await cdp.send("Page.navigate", { url: "https://grok.com/" });
    await delay(1500);

    const { cookies } = await cdp.send("Network.getCookies", {
      urls: ["https://grok.com/", "https://x.ai/", "https://accounts.x.ai/"],
    });

    const cookieMap = new Map();
    for (const cookie of cookies || []) {
      if (!cookie?.name || typeof cookie.value !== "string") continue;
      cookieMap.set(cookie.name, cookie.value);
    }

    const sso = cookieMap.get("sso");
    if (!sso) {
      fail(
        "Profile does not contain Grok session cookie `sso`. Login may be missing or session expired.",
        { profileDir },
      );
    }

    const orderedNames = Array.from(cookieMap.keys()).sort((left, right) => {
      if (left === "sso") return -1;
      if (right === "sso") return 1;
      if (left === "sso-rw") return -1;
      if (right === "sso-rw") return 1;
      if (left === "cf_clearance") return -1;
      if (right === "cf_clearance") return 1;
      return left.localeCompare(right);
    });

    const raw = orderedNames
      .map((name) => `${name}=${cookieMap.get(name)}`)
      .join("; ");

    const payload = {
      sso,
      ...(cookieMap.get("sso-rw") ? { "sso-rw": cookieMap.get("sso-rw") } : {}),
      ...(cookieMap.get("cf_clearance") ? { cf_clearance: cookieMap.get("cf_clearance") } : {}),
      _raw: raw,
    };

    for (const [name, value] of cookieMap.entries()) {
      payload[name] = value;
    }

    emit({
      ok: true,
      profileDir,
      cookies: payload,
      message: `Synced ${orderedNames.length} cookies from profile.`,
    });
  } finally {
    cdp.close();
  }
} catch (error) {
  const details = stderr.trim();
  fail(
    details ? `${String(error)} ${details}` : String(error),
    { profileDir },
  );
} finally {
  chrome.kill("SIGTERM");
}

function emit(payload) {
  process.stdout.write(JSON.stringify(payload));
}

function fail(message, extra = {}) {
  emit({ ok: false, error: message, ...extra });
  process.exit(1);
}

function resolveChromeBinary() {
  if (process.env.GOOGLE_CHROME_BIN) return process.env.GOOGLE_CHROME_BIN;

  const candidates = [
    "google-chrome",
    "google-chrome-stable",
    "chromium",
    "chromium-browser",
  ];

  for (const candidate of candidates) {
    const result = spawnSync("bash", ["-lc", `command -v ${candidate}`], {
      encoding: "utf8",
    });
    const found = result.stdout.trim();
    if (result.status === 0 && found) return found;
  }

  return null;
}

function buildChromeEnv({ requireDisplay }) {
  const env = { ...process.env };

  if (!requireDisplay) {
    return env;
  }

  if (!env.DISPLAY) {
    env.DISPLAY = ":0";
  }

  if (!env.XAUTHORITY) {
    const home = env.HOME || process.env.HOME;
    const defaultXauthority = home ? `${home}/.Xauthority` : "";
    if (defaultXauthority && existsSync(defaultXauthority)) {
      env.XAUTHORITY = defaultXauthority;
    }
  }

  return env;
}

async function waitForDebuggerTarget(port) {
  for (let attempt = 0; attempt < 40; attempt += 1) {
    try {
      const response = await fetch(`http://127.0.0.1:${port}/json/list`);
      if (response.ok) {
        const targets = await response.json();
        const pageTarget = targets.find((target) => target.type === "page" && target.webSocketDebuggerUrl);
        if (pageTarget?.webSocketDebuggerUrl) {
          return pageTarget.webSocketDebuggerUrl;
        }
      }
    } catch {
      // Wait for Chrome to finish booting.
    }

    if (chromeExited(chrome)) {
      throw new Error("Chrome exited before remote debugging became ready.");
    }

    await delay(250);
  }

  throw new Error("Timed out waiting for Chrome remote debugging endpoint.");
}

function chromeExited(child) {
  return child.exitCode !== null || child.killed;
}

async function connectCdp(wsUrl) {
  const ws = new WebSocket(wsUrl);
  await new Promise((resolve, reject) => {
    ws.addEventListener("open", resolve, { once: true });
    ws.addEventListener("error", (event) => reject(event.error || new Error("CDP websocket failed")), { once: true });
  });

  let nextId = 1;
  const pending = new Map();

  ws.addEventListener("message", (event) => {
    const payload = JSON.parse(event.data.toString());
    if (!payload.id) return;

    const handler = pending.get(payload.id);
    if (!handler) return;
    pending.delete(payload.id);

    if (payload.error) {
      handler.reject(new Error(payload.error.message || "Unknown CDP error"));
      return;
    }

    handler.resolve(payload.result || {});
  });

  ws.addEventListener("close", () => {
    for (const handler of pending.values()) {
      handler.reject(new Error("CDP websocket closed."));
    }
    pending.clear();
  });

  return {
    send(method, params = {}) {
      return new Promise((resolve, reject) => {
        const id = nextId++;
        pending.set(id, { resolve, reject });
        ws.send(JSON.stringify({ id, method, params }));
      });
    },
    close() {
      ws.close();
    },
  };
}
