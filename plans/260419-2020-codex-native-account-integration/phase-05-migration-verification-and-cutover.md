---
phase: 5
status: in-progress
priority: medium
---

# Phase 5: Migration, Verification, And Cutover

## Context Links

- `src/main.rs`
- `src/db/migrate.rs`
- `test-models.sh`
- `web/README.md`

## Overview

Khóa migration, test compatibility Grok, verify Codex flow end-to-end rồi mới bật dùng thật.

## Current Status

- Đã xong verify build/test nội bộ: `cargo build`, `cargo test`, `web/npm run build`.
- Chưa xong verify cutover ngoài môi trường local.
- Vì vậy phase này giữ `in-progress`.

## Key Insights

- Risk lớn nhất không nằm ở riêng Codex, mà ở refactor account layer làm gãy Grok đang chạy.
- Cần test cả DB migration lẫn request path thật.

## Requirements

- Migration chạy an toàn trên DB đang có account/proxy.
- Grok request cũ pass.
- Codex login, refresh, và chat stream pass.
- FE admin build pass sau refactor account UI.

## Architecture

### Verification matrix

- DB migration
- Admin accounts CRUD
- Grok chat request
- Codex OAuth login
- Codex refresh
- Codex streamed response
- Account pause/default/rotation

## Related Code Files

- Update: `src/main.rs`
- Update: test/build scripts liên quan nếu cần
- Update: docs vận hành nếu repo đã có nơi ghi runbook

## Implementation Steps

1. Chuẩn bị migration test trên snapshot DB local.
2. Chạy `cargo build` sau từng phase lớn.
3. Build frontend admin.
4. Test Grok regression.
5. Test Codex login + real request.
6. Chỉ sau khi pass mới expose model Codex cho user plan/public model list.

## Todo List

- [ ] Migration rehearsal
- [x] Backend compile verification
- [x] Frontend build verification
- [ ] Grok regression checklist
- [ ] Codex end-to-end checklist

## Success Criteria

- Không có regression Grok rõ ràng.
- Có ít nhất 1 Codex account chạy request thật end-to-end.
- Rollback path rõ: disable Codex provider, giữ Grok nguyên trạng.

## Risk Assessment

- Quota/usage-limit phía Codex có thể gây false negative khi test.
- Callback/firewall local có thể làm login fail dù code đúng.

## Security Considerations

- Thu hồi hoặc pause ngay account test lỗi auth liên tiếp.
- Không commit secrets, token captures, callback query logs.

## Next Steps

Chỉ nên coi plan hoàn tất sau khi đóng đủ các gap sau:

- chạy migration thật trên Postgres có data cũ
- chạy OAuth thật với OpenAI account thật
- bắn request Codex `/v1/responses` thật và kiểm tra Grok regression tối thiểu
