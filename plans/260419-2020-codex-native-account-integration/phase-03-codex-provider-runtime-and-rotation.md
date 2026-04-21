---
phase: 3
status: completed
priority: high
---

# Phase 3: Codex Provider Runtime And Rotation

## Context Links

- `src/providers/mod.rs`
- `src/providers/chat_provider.rs`
- `src/providers/grok_chat.rs`
- `src/account/pool.rs`
- `src/conversation.rs`

## Overview

Thêm lane runtime cho Codex và biến pool/account selection thành provider-aware.

## Completion Notes

- Provider registry đã có cả `grok` và `codex`.
- Pool/runtime đã chọn account theo `provider_slug`, ưu tiên `is_default`, và refresh Codex trước request khi cần.
- Health/rotation tracking ở request path đã chuyển sang update theo `account_id` thực tế thay vì ghi nhầm theo provider.
- Codex client dùng Responses-style SSE path native.

## Key Insights

- `ProviderRegistry` hiện chỉ `with_grok`.
- `AccountPool` đang xoay global, chưa tách theo provider.
- Codex request shape khác Grok: Responses API + SSE.

## Requirements

- Model Codex phải map đúng sang provider `codex`.
- Runtime chọn account đúng theo provider, không trộn Grok/Codex.
- Hỗ trợ default account, pause, health-based rotation.
- Refresh token trước request nếu cần.

## Architecture

### Runtime shape

- `ProviderRegistry::new(grok, codex?)`
- `ProviderAccountPool`
  - pick accounts by `provider_slug`
  - ưu tiên `is_default`
  - bỏ qua account paused/refresh_failed

### Codex provider

- `CodexChatProvider`
  - build request theo Responses API
  - dùng `Accept: text/event-stream`
  - parse SSE chunk về unified `ChatStreamEvent`

## Related Code Files

- Update: `src/providers/mod.rs`
- Update: `src/account/pool.rs`
- Update: `src/providers/types.rs`
- Update: `src/api/models.rs` hoặc module model listing liên quan
- Create: `src/providers/codex_chat.rs`
- Create: `src/services/codex_client.rs`

## Implementation Steps

1. Refactor pool để query account theo `provider_slug`.
2. Thêm selection strategy: default first, rồi healthy round-robin.
3. Viết Codex HTTP client + SSE parser.
4. Thêm Codex vào provider registry.
5. Seed provider/models Codex trong DB nếu model system đã có.
6. Map lỗi upstream thành error nội bộ rõ ràng: auth expired, usage limited, upstream timeout.

## Todo List

- [x] Provider-aware rotation
- [x] Codex client
- [x] Codex SSE parser
- [x] Model/provider registration

## Success Criteria

- Request vào model Codex đi qua đúng provider.
- Grok vẫn hoạt động song song.
- Khi một Codex account fail auth, pool có thể né account đó hoặc refresh trước khi retry.

## Risk Assessment

- Pool toàn cục nếu giữ nguyên sẽ chọn sai provider.
- Parser SSE sai sẽ làm stream đứt hoặc mất text delta.

## Security Considerations

- Không retry vô hạn với token lỗi.
- Sanitize log request/response headers của Codex.

## Next Steps

Done. Admin surface đã được nâng theo phase 4.
