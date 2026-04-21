#!/usr/bin/env node

import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import {
  evalInPage,
  hasDisplaySupport,
  launchDetachedChrome,
  navigate,
  screenshot,
  startRemoteChrome,
  stopRemoteChrome,
  waitFor,
} from "./lib/chrome-cdp-utils.mjs";

const DEFAULT_PROFILE_DIR = "data/browser-profiles/chatgpt-web-test";
const DEFAULT_OUTPUT_PATH = "tmp/chatgpt-web-model-test-results.json";
const DEFAULT_MODELS = [
  "gpt-5-codex",
  "gpt-5.2-codex",
  "gpt-5.1-codex",
  "gpt-5.1-codex-max",
  "gpt-5.1-codex-mini",
  "codex-mini-latest",
  "gpt-5",
  "gpt-5-codex-mini",
  "gpt-5.1",
  "gpt-5.3-codex-spark",
];
const MODEL_LABELS = {
  "gpt-5-codex": ["GPT-5-Codex", "gpt-5-codex"],
  "gpt-5.2-codex": ["GPT-5.2-Codex", "gpt-5.2-codex"],
  "gpt-5.1-codex": ["GPT-5.1 Codex", "GPT-5.1-Codex", "gpt-5.1-codex"],
  "gpt-5.1-codex-max": ["GPT-5.1-Codex-Max", "gpt-5.1-codex-max"],
  "gpt-5.1-codex-mini": ["GPT-5.1 Codex mini", "GPT-5.1-Codex-mini", "gpt-5.1-codex-mini"],
  "codex-mini-latest": ["codex-mini-latest"],
  "gpt-5": ["GPT-5", "gpt-5"],
  "gpt-5-codex-mini": ["GPT-5 Codex Mini", "gpt-5-codex-mini"],
  "gpt-5.1": ["GPT-5.1", "gpt-5.1"],
  "gpt-5.3-codex-spark": ["GPT-5.3 Codex Spark", "gpt-5.3-codex-spark"],
};

const [, , command, ...rest] = process.argv;
if (!command || command === "--help" || command === "-h") usage(0);

if (command === "launch-login") {
  const profileDir = rest[0] || DEFAULT_PROFILE_DIR;
  try {
    const pid = await launchDetachedChrome({ profileDir, url: "https://chatgpt.com/" });
    emit({
      ok: true,
      profileDir,
      pid,
      message: "Chrome launched. Log in to ChatGPT in that window, then close it before running test-models.",
    });
    process.exit(0);
  } catch (error) {
    fail(String(error.message || error));
  }
}

if (command !== "test-models") usage(1);

const options = parseOptions(rest);
if (!options.headed && !options.headless && !hasDisplaySupport()) {
  options.headless = true;
}
const models = options.models.length ? options.models : DEFAULT_MODELS;
const outputPath = options.output || DEFAULT_OUTPUT_PATH;
const screenshotDir = options.screenshotDir || "tmp/chatgpt-web-model-test";
await mkdir(screenshotDir, { recursive: true });

const { chrome, cdp } = await startRemoteChrome({
  profileDir: options.profileDir || DEFAULT_PROFILE_DIR,
  headless: options.headless,
  url: "https://chatgpt.com/",
});

try {
  const ready = await waitForChatReady(cdp, options.headless);
  if (!ready.ok) fail(ready.message);

  const results = [];
  for (const model of models) {
    const result = await testOneModel(cdp, model, screenshotDir);
    results.push(result);
    process.stdout.write(JSON.stringify(result) + "\n");
  }

  await writeFile(outputPath, JSON.stringify({
    testedAt: new Date().toISOString(),
    profileDir: options.profileDir || DEFAULT_PROFILE_DIR,
    models,
    results,
  }, null, 2));

  emit({ ok: true, outputPath, results });
} finally {
  await stopRemoteChrome(chrome, cdp);
}

async function testOneModel(cdp, model, screenshotDir) {
  await navigate(cdp, "https://chatgpt.com/");
  await waitForChatReady(cdp, false);
  await evalInPage(cdp, clickTextIfVisible, ["New chat", "New"]);
  const selected = await selectModel(cdp, model);
  if (!selected.ok) {
    return await finalizeFailure(cdp, screenshotDir, model, selected.reason, selected.debug);
  }

  const prompt = `Reply with exactly OK. Then stop. [model-test:${model}]`;
  const sent = await evalInPage(cdp, sendPrompt, prompt);
  if (!sent?.ok) {
    return await finalizeFailure(cdp, screenshotDir, model, "composer_not_found", sent?.debug || "");
  }

  const reply = await waitFor(cdp, waitForReplyState, { timeoutMs: 70000, intervalMs: 1500, args: [prompt] });
  if (reply?.status === "ok") {
    return { model, ok: true, status: "ok", detail: reply.detail };
  }

  return await finalizeFailure(cdp, screenshotDir, model, reply?.status || "timeout", reply?.detail || "");
}

async function selectModel(cdp, model) {
  const labels = MODEL_LABELS[model] || [model];
  const alreadySelected = await evalInPage(cdp, isModelVisibleOnTopBar, labels);
  if (alreadySelected?.ok) return { ok: true, reason: "already-selected" };

  const opened = await evalInPage(cdp, openModelPicker, labels);
  if (!opened?.ok) return opened;
  await new Promise((resolve) => setTimeout(resolve, 800));

  const chosen = await evalInPage(cdp, chooseModelOption, labels);
  if (!chosen?.ok) return chosen;
  await new Promise((resolve) => setTimeout(resolve, 1200));
  return { ok: true, reason: "selected" };
}

async function finalizeFailure(cdp, screenshotDir, model, status, detail) {
  const image = await screenshot(cdp);
  const imagePath = path.join(screenshotDir, `${model.replace(/[^a-z0-9._-]/gi, "_")}.png`);
  await writeFile(imagePath, Buffer.from(image, "base64"));
  return { model, ok: false, status, detail, screenshot: imagePath };
}

async function waitForChatReady(cdp, headless) {
  const state = await waitFor(cdp, detectChatState, { timeoutMs: headless ? 20000 : 180000, intervalMs: 1500 });
  if (state?.status === "ready") return { ok: true };
  if (state?.status === "challenge") {
    return { ok: false, message: "ChatGPT returned a Cloudflare/browser challenge. Headless mode is blocked for this profile on this server." };
  }
  if (state?.status === "login" && !headless) {
    return { ok: false, message: "ChatGPT web is not logged in for this profile. Run launch-login first and finish login." };
  }
  return { ok: false, message: state?.debug || "ChatGPT UI did not become ready." };
}

function parseOptions(argv) {
  const options = { profileDir: "", output: "", screenshotDir: "", headless: false, headed: false, models: [] };
  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    const next = argv[index + 1];
    if (token === "--profile-dir" && next) options.profileDir = next, index += 1;
    else if (token === "--output" && next) options.output = next, index += 1;
    else if (token === "--screenshot-dir" && next) options.screenshotDir = next, index += 1;
    else if (token === "--models" && next) options.models = next.split(",").map((value) => value.trim()).filter(Boolean), index += 1;
    else if (token === "--headless") options.headless = true;
    else if (token === "--headed") options.headed = true, options.headless = false;
  }
  return options;
}

function usage(exitCode) {
  process.stderr.write(
    "Usage:\n" +
    "  node scripts/chatgpt-web-model-test.mjs launch-login [profile-dir]\n" +
    "  node scripts/chatgpt-web-model-test.mjs test-models [--profile-dir dir] [--models a,b,c] [--output file] [--headless|--headed]\n" +
    "\n" +
    "Notes:\n" +
    "  - launch-login requires a visible desktop session.\n" +
    "  - test-models auto-switches to headless on Linux servers without DISPLAY.\n",
  );
  process.exit(exitCode);
}

function emit(payload) {
  process.stdout.write(JSON.stringify(payload, null, 2) + "\n");
}

function fail(message) {
  process.stderr.write(message + "\n");
  process.exit(1);
}

function detectChatState() {
  const text = document.body?.innerText || "";
  const title = document.title || "";
  if (document.querySelector("textarea, #prompt-textarea, [contenteditable='true']")) return { done: true, status: "ready" };
  if (/just a moment/i.test(title) || /verify you are human|checking your browser|cloudflare/i.test(text)) {
    return { done: true, status: "challenge", debug: `${title}\n${text}`.slice(0, 500) };
  }
  if (/log in|sign up|continue with google|continue with microsoft/i.test(text)) return { done: true, status: "login", debug: text.slice(0, 500) };
  return { done: false, status: "loading", debug: text.slice(0, 500) };
}

function isModelVisibleOnTopBar(labels) {
  const normalizeText = (value) => String(value || "").toLowerCase().replace(/\s+/g, " ").trim();
  const findClickable = () => [...document.querySelectorAll("button, [role='button'], [role='menuitem'], [role='option'], a, div[tabindex]")]
    .filter((element) => {
      const rect = element.getBoundingClientRect();
      const style = window.getComputedStyle(element);
      return rect.width > 0 && rect.height > 0 && style.visibility !== "hidden" && style.display !== "none";
    })
    .map((element) => ({
      text: [element.innerText, element.getAttribute("aria-label"), element.getAttribute("title")].filter(Boolean).join(" ").trim(),
      rect: element.getBoundingClientRect(),
    }));
  const normalized = labels.map(normalizeText);
  const clickable = findClickable().filter((item) => item.rect.top < 220);
  const match = clickable.find((item) => normalized.some((label) => normalizeText(item.text).includes(label)));
  return { ok: Boolean(match) };
}

function openModelPicker(labels) {
  const normalizeText = (value) => String(value || "").toLowerCase().replace(/\s+/g, " ").trim();
  const findClickable = () => [...document.querySelectorAll("button, [role='button'], [role='menuitem'], [role='option'], a, div[tabindex]")]
    .filter((element) => {
      const rect = element.getBoundingClientRect();
      const style = window.getComputedStyle(element);
      return rect.width > 0 && rect.height > 0 && style.visibility !== "hidden" && style.display !== "none";
    })
    .map((element) => ({
      element,
      text: [element.innerText, element.getAttribute("aria-label"), element.getAttribute("title")].filter(Boolean).join(" ").trim(),
      popup: element.getAttribute("aria-haspopup") || "",
      rect: element.getBoundingClientRect(),
    }));
  const patterns = [...labels, "model", "gpt", "o3", "o4", "ChatGPT"].map(normalizeText);
  const candidate = findClickable()
    .filter((item) => item.rect.top < 220)
    .find((item) => patterns.some((pattern) => normalizeText(item.text).includes(pattern)) || /menu|listbox|dialog/.test(item.popup));
  if (!candidate) return { ok: false, reason: "model_picker_not_found", debug: "No likely top-bar model picker found." };
  candidate.element.click();
  return { ok: true };
}

function chooseModelOption(labels) {
  const normalizeText = (value) => String(value || "").toLowerCase().replace(/\s+/g, " ").trim();
  const findClickable = () => [...document.querySelectorAll("button, [role='button'], [role='menuitem'], [role='option'], a, div[tabindex]")]
    .filter((element) => {
      const rect = element.getBoundingClientRect();
      const style = window.getComputedStyle(element);
      return rect.width > 0 && rect.height > 0 && style.visibility !== "hidden" && style.display !== "none";
    })
    .map((element) => ({
      element,
      text: [element.innerText, element.getAttribute("aria-label"), element.getAttribute("title")].filter(Boolean).join(" ").trim(),
    }));
  const normalized = labels.map(normalizeText);
  const candidate = findClickable().find((item) => normalized.some((label) => normalizeText(item.text) === label || normalizeText(item.text).includes(label)));
  if (!candidate) return { ok: false, reason: "model_option_not_found", debug: `No visible option matched: ${labels.join(", ")}` };
  candidate.element.click();
  return { ok: true };
}

function clickTextIfVisible(labels) {
  const normalizeText = (value) => String(value || "").toLowerCase().replace(/\s+/g, " ").trim();
  const findClickable = () => [...document.querySelectorAll("button, [role='button'], [role='menuitem'], [role='option'], a, div[tabindex]")]
    .filter((element) => {
      const rect = element.getBoundingClientRect();
      const style = window.getComputedStyle(element);
      return rect.width > 0 && rect.height > 0 && style.visibility !== "hidden" && style.display !== "none";
    })
    .map((element) => ({
      element,
      text: [element.innerText, element.getAttribute("aria-label"), element.getAttribute("title")].filter(Boolean).join(" ").trim(),
    }));
  const candidate = findClickable().find((item) => labels.some((label) => normalizeText(item.text).includes(normalizeText(label))));
  if (candidate) candidate.element.click();
  return { ok: true };
}

function sendPrompt(prompt) {
  const composer = document.querySelector("#prompt-textarea, textarea, [contenteditable='true']");
  if (!composer) return { ok: false, debug: "Composer not found." };
  composer.focus();
  if ("value" in composer) {
    composer.value = prompt;
    composer.dispatchEvent(new Event("input", { bubbles: true }));
  } else {
    composer.textContent = prompt;
    composer.dispatchEvent(new InputEvent("input", { bubbles: true, data: prompt }));
  }
  composer.dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, key: "Enter", code: "Enter" }));
  composer.dispatchEvent(new KeyboardEvent("keypress", { bubbles: true, key: "Enter", code: "Enter" }));
  composer.dispatchEvent(new KeyboardEvent("keyup", { bubbles: true, key: "Enter", code: "Enter" }));
  return { ok: true };
}

function waitForReplyState(prompt) {
  const text = document.body?.innerText || "";
  const lower = text.toLowerCase();
  if (/something went wrong|unable to load conversation|error generating|not available|doesn.t have access/i.test(lower)) {
    return { done: true, status: "error", detail: text.slice(-1200) };
  }
  const promptMarker = `[model-test:${prompt.match(/\[model-test:([^\]]+)\]/)?.[1] || ""}]`;
  if (text.includes(promptMarker) && /\bOK\b/.test(text.slice(-500))) {
    return { done: true, status: "ok", detail: text.slice(-500) };
  }
  return { done: false, status: "waiting", detail: text.slice(-500) };
}
