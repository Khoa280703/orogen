---
phase: 1
status: completed
priority: high
---

# Phase 1: Provider-Agnostic Account Schema

## Context Links

- `src/account/types.rs`
- `src/account/pool.rs`
- `src/db/accounts.rs`
- `src/db/migrate.rs`
- `src/api/admin_accounts.rs`

## Overview

Tách account metadata chung khỏi provider credential cụ thể. Đây là phase chặn đường, vì hiện tại runtime đang assume mọi account đều là Grok cookie account.

## Completion Notes

- Đã thêm `provider_slug`, `is_default`, `auth_mode` và lane credential riêng `account_credentials`.
- Query layer/runtime đã chuyển sang account row provider-aware thay vì assume Grok everywhere.
- Migration/seed Codex đã vào codebase, đồng thời giữ đường đọc Grok hiện có.

## Key Insights

- `accounts.cookies` đang bị overload: vừa là secret store, vừa là type contract cho Grok.
- `AccountPool` deserialize thẳng sang `GrokCookies`, nên không thể thêm Codex mà không refactor.
- Proxy binding, health, request stats là concern dùng chung cho mọi provider.

## Requirements

- Thêm `provider_slug` cho account.
- Lưu provider credential ở lane riêng, không ép mọi provider vào schema `cookies`.
- Backfill toàn bộ Grok account cũ.
- Không làm mất proxy assignment, health counters, profile dir.

## Architecture

### Data shape đề xuất

- `accounts`
  - giữ metadata runtime: `id`, `name`, `provider_slug`, `active`, `proxy_id`, `profile_dir`, `session_status`, `request_count`, health counters, timestamps
  - thêm `is_default`, `account_label`, `external_account_id`, `auth_mode`, `metadata`
- `account_credentials`
  - `account_id`
  - `credential_type`
  - `payload JSONB`
  - `updated_at`

### Credential types ban đầu

- `grok_cookies`
- `codex_oauth_tokens`

## Related Code Files

- Update: `src/db/migrate.rs`
- Update: `src/db/accounts.rs`
- Update: `src/account/types.rs`
- Update: `src/account/pool.rs`
- Update: `src/api/admin_accounts.rs`
- Create: `migrations/00x_account_credentials.sql`

## Implementation Steps

1. Thêm migration tạo `account_credentials` và cột `accounts.provider_slug`.
2. Backfill account cũ thành `provider_slug='grok'`.
3. Copy `accounts.cookies` cũ sang `account_credentials.payload` với `credential_type='grok_cookies'`.
4. Đổi query runtime từ đọc `accounts.cookies` sang join credential active.
5. Đổi type Rust sang enum/provider-aware structs thay vì `GrokCookies` everywhere.
6. Giữ compatibility đọc Grok cũ trong một release để rollback dễ.

## Todo List

- [x] Thiết kế schema migration thuận rollback
- [x] Refactor query layer cho provider-aware account rows
- [x] Refactor runtime account type
- [x] Giữ Grok pool chạy sau migration

## Success Criteria

- Có thể load Grok account cũ sau migration.
- Runtime account row mang đủ `provider_slug + credential payload`.
- Không còn chỗ nào hard fail chỉ vì account không phải Grok.

## Risk Assessment

- Migration sai dễ làm mất đường đọc account cũ.
- Refactor pool sai dễ làm Grok chết trước khi Codex vào.

## Security Considerations

- Credential payload phải tách khỏi response DTO admin mặc định.
- Không log raw token/cookie.

## Next Steps

Done. Phase này đã mở đường cho OAuth/token lane của Codex.
