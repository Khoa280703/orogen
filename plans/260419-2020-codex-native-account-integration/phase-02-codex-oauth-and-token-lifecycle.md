---
phase: 2
status: completed
priority: high
---

# Phase 2: Codex OAuth And Token Lifecycle

## Context Links

- `src/api/mod.rs`
- `src/api/admin_accounts.rs`
- `src/main.rs`
- CCS reference: `~/.ccs/cliproxy/auth/*.json`

## Overview

Xây lane login/refresh/revoke cho Codex account. Đây là phần thay CCS runtime.

## Completion Notes

- Đã thêm config OAuth, service `codex_oauth`, callback backend, refresh path và persist token bundle thật.
- `refresh_token` hiện được giữ lại khi upstream refresh response không trả field này.
- Callback page đã escape nội dung hiển thị để tránh reflected XSS.

## Key Insights

- CCS dùng Authorization Code flow qua `auth.openai.com`.
- Token bundle thực tế có cả `access_token` và `refresh_token`.
- Refresh không được CCS expose cho app ngoài; app phải tự viết ownership của refresh.

## Requirements

- Admin có thể bắt đầu login Codex từ UI.
- Backend tạo `state`, auth URL, và nhận callback.
- Exchange `code` lấy token bundle rồi persist vào `account_credentials`.
- Có refresh path trước request hoặc background refresh.
- Có trạng thái account rõ: `needs_login`, `healthy`, `refresh_failed`, `paused`.

## Architecture

### Backend endpoints đề xuất

- `POST /admin/accounts/codex/start-login`
- `GET /admin/accounts/codex/callback`
- `POST /admin/accounts/:id/refresh`
- `POST /admin/accounts/:id/set-default`

### Token payload tối thiểu

- `access_token`
- `refresh_token`
- `id_token`
- `account_id`
- `email`
- `expires_at`
- `last_refresh_at`
- `token_type='codex_oauth_tokens'`

### Callback strategy

- Ưu tiên callback vào chính backend Axum.
- Nếu OpenAI app bắt buộc loopback riêng, thêm config `CODEX_OAUTH_CALLBACK_PORT`, default `1455`.
- State + PKCE phải được backend quản lý.

## Related Code Files

- Update: `src/api/mod.rs`
- Update: `src/api/admin_accounts.rs`
- Update: `src/main.rs`
- Create: `src/services/codex_oauth.rs`
- Create: `src/services/codex_token_store.rs`

## Implementation Steps

1. Tạo config/env cho client id, callback URL/port, scopes.
2. Thêm service generate auth URL + state + PKCE verifier.
3. Thêm callback handler exchange code lấy token.
4. Persist token bundle vào credential store.
5. Viết refresh service cập nhật `access_token` khi gần expire.
6. Gắn session/account status để UI biết cần login lại hay không.

## Todo List

- [x] Xác định config surface cho OAuth
- [x] Viết state store chống replay
- [x] Viết code exchange + refresh
- [x] Ẩn token khỏi admin response

## Success Criteria

- Một Codex account login thành công từ admin UI.
- Token lưu được và refresh thành công mà không cần CCS.
- Callback lỗi hiển thị rõ lý do, không nuốt lỗi.

## Risk Assessment

- Sai callback design sẽ làm flow login không hoàn tất.
- Sai refresh handling sẽ gây 401/429 giả khi runtime dùng token cũ.

## Security Considerations

- PKCE + CSRF state bắt buộc.
- Token mã hóa at-rest nếu repo đã có secret encryption layer; nếu chưa có, ít nhất không expose qua API/list view.

## Next Steps

Done. Provider runtime cho Codex đã có thể consume lane token này.
