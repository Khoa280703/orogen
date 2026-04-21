import { existsSync } from "node:fs";
import { spawn, spawnSync } from "node:child_process";
import { mkdir } from "node:fs/promises";
import { setTimeout as delay } from "node:timers/promises";

export async function ensureProfileDir(profileDir) {
  await mkdir(profileDir, { recursive: true });
}

export function resolveChromeBinary() {
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

export function buildChromeEnv({ requireDisplay }) {
  const env = { ...process.env };

  if (!requireDisplay) return env;

  if (!env.DISPLAY) env.DISPLAY = ":0";
  if (!env.XAUTHORITY) {
    const home = env.HOME || process.env.HOME;
    const xauthority = home ? `${home}/.Xauthority` : "";
    if (xauthority && existsSync(xauthority)) env.XAUTHORITY = xauthority;
  }

  return env;
}

export function hasDisplaySupport() {
  if (process.env.DISPLAY || process.env.WAYLAND_DISPLAY) return true;
  return existsSync("/tmp/.X11-unix/X0");
}

export function findProfileLockOwner(profileDir) {
  const lockPath = `${profileDir}/SingletonLock`;
  if (!existsSync(lockPath)) return null;

  const result = spawnSync("bash", ["-lc", `readlink '${lockPath}'`], {
    encoding: "utf8",
  });
  const target = result.stdout.trim();
  if (!target) return { raw: "" };

  const match = target.match(/-(\d+)$/);
  const pid = match ? Number(match[1]) : null;
  if (pid) {
    const alive = spawnSync("bash", ["-lc", `kill -0 ${pid} >/dev/null 2>&1`], { encoding: "utf8" });
    if (alive.status !== 0) {
      return null;
    }
  }
  return {
    raw: target,
    pid,
  };
}

export async function launchDetachedChrome({ profileDir, url }) {
  const chromeBin = resolveChromeBinary();
  if (!chromeBin) throw new Error("Cannot find Chrome/Chromium binary. Set GOOGLE_CHROME_BIN.");
  if (!hasDisplaySupport()) {
    throw new Error("No DISPLAY/WAYLAND_DISPLAY available. launch-login needs a visible desktop session.");
  }

  await ensureProfileDir(profileDir);

  const child = spawn(
    chromeBin,
    [
      `--user-data-dir=${profileDir}`,
      "--no-first-run",
      "--no-default-browser-check",
      "--new-window",
      url,
    ],
    {
      detached: true,
      stdio: "ignore",
      env: buildChromeEnv({ requireDisplay: true }),
    },
  );

  child.unref();
  return child.pid;
}

export async function startRemoteChrome({ profileDir, headless = false, url = "about:blank" }) {
  const chromeBin = resolveChromeBinary();
  if (!chromeBin) throw new Error("Cannot find Chrome/Chromium binary. Set GOOGLE_CHROME_BIN.");
  if (!headless && !hasDisplaySupport()) {
    throw new Error("No DISPLAY/WAYLAND_DISPLAY available. Use headless mode for test-models on this server.");
  }

  const lockOwner = findProfileLockOwner(profileDir);
  if (lockOwner?.pid) {
    throw new Error(`Chrome profile is locked by PID ${lockOwner.pid}. Close that Chrome instance before running tests.`);
  }

  await ensureProfileDir(profileDir);

  const port = 9222 + Math.floor(Math.random() * 1000);
  const args = [
    `--user-data-dir=${profileDir}`,
    "--no-first-run",
    "--no-default-browser-check",
    "--remote-debugging-address=127.0.0.1",
    `--remote-debugging-port=${port}`,
    url,
  ];

  if (headless) {
    args.splice(1, 0, "--headless=new", "--disable-gpu");
  }

  const chrome = spawn(chromeBin, args, {
    stdio: ["ignore", "ignore", "pipe"],
    env: buildChromeEnv({ requireDisplay: !headless }),
  });

  let stderr = "";
  chrome.stderr?.on("data", (chunk) => {
    stderr += chunk.toString();
  });

  const pageWsUrl = await waitForDebuggerTarget(chrome, port);
  const cdp = await connectCdp(pageWsUrl);
  await cdp.send("Page.enable");
  await cdp.send("Runtime.enable");

  return { chrome, cdp, stderrRef: () => stderr };
}

async function waitForDebuggerTarget(child, port) {
  for (let attempt = 0; attempt < 60; attempt += 1) {
    try {
      const response = await fetch(`http://127.0.0.1:${port}/json/list`);
      if (response.ok) {
        const targets = await response.json();
        const pageTarget = targets.find((target) => target.type === "page" && target.webSocketDebuggerUrl);
        if (pageTarget?.webSocketDebuggerUrl) return pageTarget.webSocketDebuggerUrl;
      }
    } catch {
      // Wait for Chrome to boot.
    }

    if (child.exitCode !== null || child.killed) {
      throw new Error("Chrome exited before remote debugging became ready.");
    }
    await delay(250);
  }

  throw new Error("Timed out waiting for Chrome remote debugging endpoint.");
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

  return {
    send(method, params = {}) {
      const id = nextId++;
      ws.send(JSON.stringify({ id, method, params }));
      return new Promise((resolve, reject) => pending.set(id, { resolve, reject }));
    },
    close() {
      ws.close();
    },
  };
}

export async function navigate(cdp, url) {
  await cdp.send("Page.navigate", { url });
  await delay(1200);
}

export async function evalInPage(cdp, fn, ...args) {
  const expression = `(${fn.toString()})(...${JSON.stringify(args)})`;
  const result = await cdp.send("Runtime.evaluate", {
    expression,
    awaitPromise: true,
    returnByValue: true,
  });
  return result.result?.value;
}

export async function waitFor(cdp, predicate, { timeoutMs = 30000, intervalMs = 1000, args = [] } = {}) {
  const startedAt = Date.now();
  while (Date.now() - startedAt < timeoutMs) {
    const value = await evalInPage(cdp, predicate, ...args);
    if (value?.done) return value;
    await delay(intervalMs);
  }
  return await evalInPage(cdp, predicate, ...args);
}

export async function screenshot(cdp) {
  const { data } = await cdp.send("Page.captureScreenshot", { format: "png" });
  return data;
}

export async function stopRemoteChrome(chrome, cdp) {
  try {
    cdp?.close();
  } finally {
    chrome?.kill("SIGTERM");
  }
}
