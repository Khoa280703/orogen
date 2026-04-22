#!/usr/bin/env node

import { mkdir } from "node:fs/promises";
import { existsSync } from "node:fs";
import { spawn, spawnSync } from "node:child_process";
import { setTimeout as delay } from "node:timers/promises";

const [, , command, rawProfileDir, rawTargetUrl, rawProxyUrl] = process.argv;

if (!command || !rawProfileDir) {
  fail("Usage: node scripts/grok-profile-session.mjs <launch-login|sync-cookies> <profile-dir> [target-url] [proxy-url]");
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
  const targetUrl = (rawTargetUrl || "https://grok.com/").trim() || "https://grok.com/";
  const proxyUrl = (rawProxyUrl || "").trim();
  const launchArgs = [
    `--user-data-dir=${profileDir}`,
    "--no-first-run",
    "--no-default-browser-check",
    "--new-window",
  ];
  if (proxyUrl) {
    launchArgs.push(`--proxy-server=${normalizeChromeProxy(proxyUrl)}`);
  }
  launchArgs.push(targetUrl);

  const child = spawn(
    chromeBin,
    launchArgs,
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
    message: proxyUrl
      ? "Browser launched with assigned proxy. Finish login in that window, then close it before syncing cookies."
      : "Browser launched. Finish login in that window, then close it before syncing cookies.",
  });
  process.exit(0);
}

if (command === "probe-proxy") {
  const targetUrl = (rawTargetUrl || "https://api.ipify.org/?format=json").trim() || "https://api.ipify.org/?format=json";
  const proxyUrl = (rawProxyUrl || "").trim();
  if (!proxyUrl) {
    fail("Proxy URL is required for probe-proxy.", { profileDir });
  }

  const probeResult = await runHeadlessChrome({
    profileDir,
    proxyUrl,
    navigateUrl: targetUrl,
    handler: async (cdp) => {
      const payload = await readBodyText(cdp);
      const observedIp = extractIpAddress(payload);
      if (!observedIp) {
        throw new Error(`Could not determine exit IP from response: ${payload.slice(0, 200)}`);
      }
      return {
        observedIp,
        observedBody: payload,
      };
    },
  });

  emit({
    ok: true,
    profileDir,
    observedIp: probeResult.observedIp,
    observedBody: probeResult.observedBody,
    message: `Proxy browser probe succeeded. Exit IP: ${probeResult.observedIp}`,
  });
  process.exit(0);
}

if (command !== "sync-cookies") {
  fail(`Unknown command: ${command}`);
}

try {
  const syncResult = await runHeadlessChrome({
    profileDir,
    navigateUrl: "https://grok.com/",
    handler: async (cdp) => {
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
        throw new Error("Profile does not contain Grok session cookie `sso`. Login may be missing or session expired.");
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

      return {
        cookies: payload,
        orderedCount: orderedNames.length,
      };
    },
  });

  emit({
    ok: true,
    profileDir,
    cookies: syncResult.cookies,
    message: `Synced ${syncResult.orderedCount} cookies from profile.`,
  });
} catch (error) {
  fail(String(error), { profileDir });
}

function emit(payload) {
  process.stdout.write(JSON.stringify(payload));
}

function fail(message, extra = {}) {
  emit({ ok: false, error: message, ...extra });
  process.exit(1);
}

function normalizeChromeProxy(proxyUrl) {
  return proxyUrl.trim().replace(/\/+$/, "");
}

async function runHeadlessChrome({ profileDir, navigateUrl, proxyUrl = "", handler }) {
  const port = 9222 + Math.floor(Math.random() * 1000);
  const args = [
    `--user-data-dir=${profileDir}`,
    "--headless=new",
    "--disable-gpu",
    "--no-first-run",
    "--no-default-browser-check",
    "--remote-debugging-address=127.0.0.1",
    `--remote-debugging-port=${port}`,
  ];
  if (proxyUrl.trim()) {
    args.push(`--proxy-server=${normalizeChromeProxy(proxyUrl)}`);
  }
  args.push("about:blank");

  const chrome = spawn(
    chromeBin,
    args,
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
    const pageWsUrl = await waitForDebuggerTarget(port, chrome);
    const cdp = await connectCdp(pageWsUrl);

    try {
      await cdp.send("Page.enable");
      await cdp.send("Network.enable");
      await cdp.send("Page.navigate", { url: navigateUrl });
      await delay(1800);
      return await handler(cdp);
    } finally {
      cdp.close();
    }
  } catch (error) {
    const details = stderr.trim();
    throw new Error(details ? `${String(error)} ${details}` : String(error));
  } finally {
    chrome.kill("SIGTERM");
  }
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

async function waitForDebuggerTarget(port, chromeProcess) {
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

    if (chromeExited(chromeProcess)) {
      throw new Error("Chrome exited before remote debugging became ready.");
    }

    await delay(250);
  }

  throw new Error("Timed out waiting for Chrome remote debugging endpoint.");
}

async function readBodyText(cdp) {
  for (let attempt = 0; attempt < 20; attempt += 1) {
    const payload = await cdp.send("Runtime.evaluate", {
      expression: "document.body ? document.body.innerText : ''",
      returnByValue: true,
      awaitPromise: true,
    });
    const value = String(payload?.result?.value || "").trim();
    if (value) {
      return value;
    }
    await delay(250);
  }

  return "";
}

function extractIpAddress(payload) {
  if (!payload) return null;

  const jsonMatch = payload.match(/\"ip\"\\s*:\\s*\"([^\"]+)\"/i);
  if (jsonMatch?.[1]) {
    return jsonMatch[1];
  }

  const textMatch = payload.match(/\b(?:\d{1,3}\.){3}\d{1,3}\b/);
  if (textMatch?.[0]) {
    return textMatch[0];
  }

  return null;
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
