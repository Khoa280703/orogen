# Phase 2: Multi API Keys

## Overview
- Priority: High
- Status: complete
- Support nhiều API keys cho nhiều khách hàng

## Requirements
- Load danh sách API keys từ config.json
- Auth middleware check Bearer token against key list
- Per-key request counting (in-memory, log to stdout)
- Backward compat: single `apiToken` vẫn hoạt động

## Architecture
```
config.json:
  "apiToken": "master-key"        ← backward compat
  "apiKeys": ["key-1", "key-2"]   ← multi-key support
  
Both merged into single HashSet for lookup.
Empty = no auth required.
```

## Files to Modify
- `src/config.rs` — add `api_keys: Vec<String>` field
- `src/api/mod.rs` — update auth_middleware to check against merged key set
- `src/api/mod.rs` — add per-key request counter (Arc<RwLock<HashMap<String, u64>>>)
- `src/main.rs` — log active key count on startup

## Implementation Steps
1. Add `api_keys: Vec<String>` to AppConfig
2. Create helper `AppConfig::all_keys() -> HashSet<String>` merging apiToken + apiKeys
3. Update auth_middleware to use all_keys()
4. Add request counter to AppState, increment per request
5. Add `GET /admin/stats` endpoint — returns per-key usage counts (protected by master key)
6. Test: multiple keys, invalid key rejected, stats endpoint works

## Success Criteria
- Multiple API keys accepted
- Invalid keys rejected with 401
- Per-key counter tracks usage
- Single apiToken backward compat preserved
