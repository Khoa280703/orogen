---
title: "Anthropic-Compatible Endpoint For Claude Code"
description: "Hoan thien endpoint Anthropic-compatible tren backend Grok hien co de Claude Code goi duoc."
status: pending
priority: P1
effort: 2h
branch: n/a
tags: [anthropic, claude-code, grok, api]
created: 2026-04-08
---

# Anthropic-Compatible Endpoint Plan

## Scope
- Ho tro endpoint Anthropic-compatible de Claude Code goi vao `duanai`.
- Reuse logic Grok hien co, khong them provider layer moi.

## Current State
- [src/api/mod.rs](/home/khoa2807/working-sources/duanai/src/api/mod.rs) da mount `POST /v1/messages` va `POST /v1/messages/count_tokens`.
- [src/api/anthropic_messages.rs](/home/khoa2807/working-sources/duanai/src/api/anthropic_messages.rs) da co translate co ban request/response, nhung moi xu ly text blocks va SSE toi thieu.
- [src/api/chat_completions.rs](/home/khoa2807/working-sources/duanai/src/api/chat_completions.rs) dang giu shared retry/usage helpers cho ca OpenAI route va Anthropic route.

## Files To Modify
- [src/api/mod.rs](/home/khoa2807/working-sources/duanai/src/api/mod.rs)
  - Giu `/v1/messages`; them alias `/messages` chi neu Claude Code khong goi qua `/v1`.
- [src/api/anthropic_messages.rs](/home/khoa2807/working-sources/duanai/src/api/anthropic_messages.rs)
  - Hoan thien flatten request Anthropic.
  - Chuan hoa non-stream response va SSE events cho Claude clients.
- [src/api/chat_completions.rs](/home/khoa2807/working-sources/duanai/src/api/chat_completions.rs)
  - Tach/giu shared Grok gateway helpers neu can de tranh duplicate.
- [src/main.rs](/home/khoa2807/working-sources/duanai/src/main.rs)
  - Cap nhat startup log de phan anh protocol/model support.
- [src/api/models.rs](/home/khoa2807/working-sources/duanai/src/api/models.rs)
  - Chi sua neu can dong bo model listing/alias.

## Implementation Steps
1. Xac dinh contract Claude Code thuc te can: path, headers, `anthropic-version`, stream behavior.
2. Normalize Anthropic request sang `GrokRequest`: merge `system`, flatten text blocks, xu ly ro unsupported blocks.
3. Map Grok response ve format Anthropic:
   - non-stream `message`
   - stream `message_start` -> `content_block_*` -> `message_delta` -> `message_stop`
4. Chi them compatibility route ngoai `/v1` neu runtime verify cho thay can.
5. Giu auth compatibility voi `x-api-key` va Bearer.

## Verify Checklist
- `cargo check`
- `curl` non-stream: `POST /v1/messages` voi `x-api-key` + `anthropic-version`
- `curl` stream: `POST /v1/messages` voi `"stream": true`
- Neu co alias, verify them `POST /messages`
- Verify end-to-end tu Claude Code/CCS voi model `grok-4`
- Xac nhan `401` chi xay ra khi key sai, khong phai key hop le

## Done When
- Claude-style request vao duoc backend va tra loi dung format Anthropic
- Claude-style streaming khong vo parser
- Khong gay regression cho `/v1/chat/completions`

## Unresolved Questions
- Claude Code dang goi `/v1/messages` hay `/messages` khi dung custom backend?
- Claude Code co can field Anthropic nao chat hon payload toi thieu hien tai khong?
