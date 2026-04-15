# Brainstorm: Scale thương mại — Bypass CF không cần browser

## Problem
Kiến trúc hiện tại (1 Python daemon + 1 Chrome instance) chỉ handle ~1-3 concurrent users. Cần scale cho thương mại.

## Research: grok2api-rs
- wreq `6.0.0-rc.27` + `Emulation::Chrome136` bypass CF thành công, KHÔNG cần browser/cf_clearance
- Fresh client per request (TLS handshake mới mỗi lần)
- Headers match Chrome 136 chính xác
- x-statsig-id random (base64 fake JS errors)
- Token selection: highest-quota-first, không round-robin
- Background quota sync qua Grok rate-limits API

## Decision
Bỏ Python daemon → wreq thuần Rust. Giữ daemon script làm optional fallback.

## Files to change
- `Cargo.toml` — thêm wreq, wreq-util
- `src/grok/client.rs` — viết lại, bỏ daemon
- `src/grok/headers.rs` — Chrome 136 headers + random statsig
- `src/account/pool.rs` — quota-based selection
- `src/cli_chat.rs` — điều chỉnh streaming

## Risks
- wreq vẫn RC, chưa stable
- CF có thể update fingerprint database
- Cùng UA cho mọi request → fingerprint risk ở scale lớn
