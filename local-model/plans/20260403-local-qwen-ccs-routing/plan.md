---
title: "Integrate local Qwen3.5 vLLM into Claude Code/CCS"
description: "Route Claude Code or CCS to the local vLLM endpoint at 127.0.0.1:8002 with the least-hacky path first."
status: pending
priority: P2
effort: 1h
branch: n/a
tags: [qwen, vllm, ccs, claude-code, local-routing]
created: 2026-04-03
---

# Plan

## Goal
Use `Claude Code` via `CCS` or native persisted settings to route requests to local `Qwen3.5` at `http://127.0.0.1:8002`, ưu tiên ít hack nhất.

## Current facts
- `vLLM` at `8002` exposes `/v1/models`, `/v1/messages`, `/v1/messages/count_tokens`.
- Live check passed: model `qwen3.5-27b`, `max_model_len=190000`.
- `count_tokens` works.
- `POST /v1/messages` responds, but current quick sample stopped at `max_tokens` and returned only `thinking`, so stream/final-shape still needs validation against Claude Code expectations.

## Chosen approach
1. Try direct `CCS API profile -> vLLM /v1/messages`.
2. Persist that profile into `~/.claude/settings.json` with `ccs persist`.
3. Only if direct compatibility fails, add a very small local adapter that normalizes Anthropic responses/streaming for Claude Code.

## Implementation steps
1. Validate compatibility.
   - Test `/v1/messages` non-stream and `stream=true`.
   - Confirm final assistant text appears, not only `thinking`.
   - Confirm headers/auth can be dummy and Claude Code accepts returned schema.
2. Create `CCS` profile `qwen-local`.
   - Add `~/.ccs/qwen-local.settings.json`.
   - Set `ANTHROPIC_BASE_URL=http://127.0.0.1:8002`.
   - Set `ANTHROPIC_AUTH_TOKEN=dummy`.
   - Set `ANTHROPIC_MODEL=qwen3.5-27b`.
   - Mirror default model vars to `qwen3.5-27b` if CCS expects them.
   - Register profile in `~/.ccs/config.yaml`.
3. Verify through `CCS`.
   - Run `ccs env qwen-local --format raw`.
   - Run `ccs persist qwen-local --yes`.
   - Start Claude Code with persisted config and send simple prompts.
4. Fallback only if needed.
   - If direct `/v1/messages` stream/tool-use shape is incompatible, add a thin proxy in this workspace that maps Claude/Anthropic-style requests to upstream vLLM and fixes SSE/event framing only.

## Verify
- `curl http://127.0.0.1:8002/v1/models` shows `qwen3.5-27b`.
- `curl ... /v1/messages/count_tokens` returns token count.
- `curl ... /v1/messages` returns usable final answer.
- `ccs env qwen-local --format anthropic` prints `8002`.
- After `ccs persist qwen-local --yes`, Claude Code answers from local Qwen and no longer hits `8317`.
- Ask `bạn là model gì`; expected answer is `Qwen3.5-27B` or equivalent local identity from system prompt.

## Notes
- Least-hacky path is direct `CCS API profile -> vLLM`.
- Highest risk is Anthropic streaming/tool-use compatibility, not auth or routing.
