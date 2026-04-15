# Phase 1: Proxy Pool

## Overview
- Priority: High
- Status: complete
- Thêm support nhiều US proxy, round-robin rotation, bind proxy per account

## Requirements
- Load danh sách proxy từ config.json
- Mỗi account bind cố định 1 proxy (tránh CF flag account nhảy IP)
- Fallback: nếu proxy chết, rotate sang proxy khác
- CLI chat vẫn hoạt động bình thường

## Architecture
```
config.json:
  "proxies": ["socks5h://user:pass@ip1:port", "socks5h://user:pass@ip2:port"]
  "proxy": "socks5h://..."  ← backward compat, treated as single-item list

AccountPool assigns proxy to each account on load:
  account[0] → proxy[0]
  account[1] → proxy[1]
  account[2] → proxy[0]  ← wraps around
```

## Files to Modify
- `src/config.rs` — add `proxies: Vec<String>` field, merge with existing `proxy`
- `src/account/types.rs` — add `proxy_url: Option<String>` to AccountEntry
- `src/account/pool.rs` — assign proxy to accounts on load (round-robin)
- `src/grok/client.rs` — `build_client()` accepts proxy_url parameter instead of reading from self
- `src/grok/client.rs` — `send_request()` and `send_request_stream()` accept proxy_url
- `src/api/chat_completions.rs` — pass account's proxy to grok client

## Implementation Steps
1. Add `proxies: Vec<String>` to AppConfig, merge `proxy` into list
2. Add `proxy_url: Option<String>` to AccountEntry (not serialized to cookies.json)
3. In AccountPool::new(), assign proxy round-robin to each account
4. Refactor GrokClient to accept proxy_url per-request instead of storing globally
5. Update chat_completions and cli_chat to pass proxy from account
6. Test: multiple proxies in config, verify different accounts use different proxies

## Success Criteria
- Multiple proxies loaded from config
- Each account bound to specific proxy
- Requests go through correct proxy per account
- Backward compat: single `proxy` field still works
