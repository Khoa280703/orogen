---
phase: 4
status: completed
priority: medium
---

# Phase 4: Admin API And UI For Codex Accounts

## Context Links

- `src/api/admin_accounts.rs`
- `web/src/app/admin/accounts/page.tsx`
- `web/src/lib/api.ts`

## Overview

Nâng admin accounts từ Grok-only thành multi-provider account management.

## Completion Notes

- Admin API/UI đã chuyển sang multi-provider DTO và provider-specific actions cho Grok/Codex.
- Codex account có start login, refresh token, set default; Grok vẫn giữ các action profile/browser cũ.
- UI không render raw token Codex.
- Follow-up nhỏ còn lại: dialog edit hiện chưa sync badge `refresh_failed` ngay trong modal nếu manual refresh fail; list state đã reload nhưng modal có thể cần đóng/mở lại để thấy badge mới.

## Key Insights

- UI hiện validate `sso` cứng.
- Action hiện có chỉ hợp với Grok browser profile: `Open Browser`, `Sync Profile`.
- Codex cần UX khác: `Start Login`, `Refresh Token`, `Set Default`, `Pause/Resume`.

## Requirements

- Cho phép chọn provider khi tạo account.
- Form fields thay đổi theo provider.
- List view hiển thị provider, email/account_id, default, health, auth status.
- Không lộ raw token ở UI.

## Architecture

### Admin actions

- Grok
  - Create/update raw cookies
  - Open login browser
  - Sync profile
- Codex
  - Start OAuth login
  - Refresh token
  - Set default
  - Pause/resume

## Related Code Files

- Update: `src/api/admin_accounts.rs`
- Update: `src/api/mod.rs`
- Update: `web/src/app/admin/accounts/page.tsx`
- Update: `web/src/lib/api.ts`

## Implementation Steps

1. Mở rộng DTO admin account để có `provider_slug`, `account_label`, `is_default`, `auth_status`.
2. Đổi list/create/update API sang provider-aware.
3. Refactor UI form thành dynamic sections cho Grok và Codex.
4. Thêm action buttons riêng cho Codex.
5. Giữ Grok UX cũ hoạt động, chỉ ẩn action không phù hợp theo provider.

## Todo List

- [x] Provider selector trong dialog
- [x] Provider-specific validation
- [x] Provider-specific actions
- [x] Hiển thị auth/account health rõ ràng

## Success Criteria

- Admin tạo được account Grok và Codex từ cùng màn hình.
- UI không còn assume mọi account đều có `sso`.
- Token Codex không bao giờ được render về textarea/list view.

## Risk Assessment

- Nếu reuse DTO cũ quá nhiều, UI dễ xuất hiện state lỗi giữa Grok/Codex.
- Nếu response admin lộ payload thô, rủi ro bảo mật cao.

## Security Considerations

- Mask email/account id khi cần.
- Không expose `refresh_token`, `access_token`, `id_token`.

## Next Steps

Done ở mức implementation. Cutover thật còn phụ thuộc phase 5 verification.
