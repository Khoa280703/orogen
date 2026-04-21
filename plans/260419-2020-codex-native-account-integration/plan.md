---
status: in-progress
created: 2026-04-19
slug: codex-native-account-integration
---

# Codex Native Account Integration

## Context

duanai đang chạy account/runtime gần như chỉ dành cho Grok:
- Backend account schema đang assume `cookies.sso` và session browser Grok.
- Pool xoay account đang load `GrokCookies` trực tiếp.
- Provider registry mới có `grok`.
- Admin UI accounts đang validate payload theo Grok cookie string/JSON.

Mục tiêu mới:
- Tích hợp `Codex` native trong app, không phụ thuộc CCS runtime.
- Flow add account phải giống CCS về bản chất: OAuth account thật, nhiều account, pause/default/rotation.
- Giữ Grok đang chạy ổn, không làm gãy API và admin hiện có.

## Confirmed From CCS

- Codex account trong CCS là OAuth account thật, không phải fake profile.
- OAuth flow là Authorization Code.
- Auth URL gốc: `https://auth.openai.com/authorize`
- CCS dùng callback local port `1455`.
- Chat/runtime gọi Codex qua Responses-style SSE endpoint.
- Token bundle có `access_token`, `refresh_token`, `id_token`, `account_id`, `email`, `expired`, `last_refresh`, `type`.
- Refresh hiện do CLIProxy sở hữu. Nếu làm native trong app, app phải tự refresh.

## Decisions

- Không đọc trực tiếp `~/.ccs` trong production flow.
- Reuse pattern của CCS, nhưng ownership của OAuth/token nằm hoàn toàn trong duanai.
- Refactor account layer thành provider-agnostic trước khi thêm Codex runtime.
- Giữ backward compatibility cho Grok bằng migration chuyển `cookies` cũ sang credential lane mới.
- Callback OAuth sẽ dùng local HTTP endpoint trong backend, mặc định port app hiện tại; nếu cần loopback port riêng thì giữ `1455` làm default cấu hình.

## Phases

| Phase | File | Status |
|-------|------|--------|
| Phase 1 | [phase-01-provider-agnostic-account-schema.md](phase-01-provider-agnostic-account-schema.md) | Completed |
| Phase 2 | [phase-02-codex-oauth-and-token-lifecycle.md](phase-02-codex-oauth-and-token-lifecycle.md) | Completed |
| Phase 3 | [phase-03-codex-provider-runtime-and-rotation.md](phase-03-codex-provider-runtime-and-rotation.md) | Completed |
| Phase 4 | [phase-04-admin-api-and-ui-for-codex-accounts.md](phase-04-admin-api-and-ui-for-codex-accounts.md) | Completed |
| Phase 5 | [phase-05-migration-verification-and-cutover.md](phase-05-migration-verification-and-cutover.md) | In Progress |

## Target Architecture

```text
Admin UI
  -> start Codex login
  -> backend creates OAuth state + auth URL
  -> user completes OpenAI auth
  -> backend callback exchanges code for tokens
  -> backend stores provider-specific credential bundle

Consumer/API request
  -> resolve model/provider
  -> pick eligible account from provider-aware pool
  -> ensure token fresh
  -> route to Codex Responses API or Grok client
  -> update account health / usage / rotation stats
```

## Done Definition

1. Done: Admin đã add/update/pause/resume/set default được cho Grok và Codex trong cùng surface.
2. Done: Codex account đã lưu token bundle thật, có refresh path native và preserve `refresh_token` khi upstream không trả lại.
3. Done: Request tới model Codex đã đi qua provider registry/runtime native, không phụ thuộc CCS runtime.
4. Done ở mức code/build: Grok flow đã được giữ tương thích sau refactor provider-aware.
5. Done: `cargo build`, `cargo test`, và `web/npm run build` đã pass.

## Current Status

- Phần implementation chính đã xong.
- Cutover thật vẫn chưa đóng, nên plan tổng thể giữ `in-progress`.
- Lý do giữ mở: còn thiếu verify ngoài môi trường local/build trước khi coi là production-ready.

## Residual External Verification Gaps

- Chưa chạy migration thật trên Postgres đang có data cũ để verify `migrations/005_codex_accounts.sql`.
- Chưa chạy OAuth Authorization Code thật với OpenAI account thật.
- Chưa bắn request thật lên Codex `/v1/responses` để xác nhận end-to-end runtime.
