# Phase 2: Rust DB Layer

## Overview
- Priority: High
- Status: complete
- Thay thế JSON file storage bằng PostgreSQL queries

## Requirements
- Database connection pool (sqlx::PgPool) trong AppState
- CRUD functions cho accounts, proxies, api_keys
- AccountPool đọc từ DB thay vì cookies.json
- API key validation query DB thay vì HashSet in-memory
- Giữ fallback JSON nếu DATABASE_URL không set

## Architecture
```
src/db/
├── mod.rs          — PgPool init, module exports
├── accounts.rs     — Account CRUD queries
├── proxies.rs      — Proxy CRUD queries
├── api_keys.rs     — API key CRUD queries
└── usage_logs.rs   — Usage log insert + query
```

## Files to Create
- `src/db/mod.rs` — init pool, re-exports
- `src/db/accounts.rs` — list, create, update, delete, get_active_with_proxy
- `src/db/proxies.rs` — list, create, update, delete
- `src/db/api_keys.rs` — list, create, revoke, validate_key
- `src/db/usage_logs.rs` — log_request, get_stats_by_key, get_stats_by_account

## Files to Modify
- `src/main.rs` — init PgPool, add to AppState
- `src/account/pool.rs` — load from DB, mark_used/mark_failure write to DB
- `src/api/mod.rs` — auth_middleware query DB for key validation
- `src/api/chat_completions.rs` — log usage to DB

## Implementation Steps
1. Create `src/db/mod.rs` with `init_pool(database_url)` → PgPool
2. Implement account queries (sqlx::query_as! with DbAccount struct)
3. Implement proxy queries
4. Implement api_key queries (validate_key returns bool)
5. Implement usage_log insert
6. Refactor AccountPool::new() to load from DB
7. Refactor auth_middleware to query DB
8. Add PgPool to AppState, init in main.rs
9. Keep JSON fallback: if no DATABASE_URL, use current JSON logic

## Key Types
```rust
// src/db/accounts.rs
struct DbAccount {
    id: i32,
    name: String,
    cookies: serde_json::Value,  // JSONB
    active: bool,
    proxy_id: Option<i32>,
    request_count: i64,
    fail_count: i32,
    success_count: i64,
    last_used: Option<chrono::DateTime<chrono::Utc>>,
}

// src/db/api_keys.rs  
struct DbApiKey {
    id: i32,
    key: String,
    label: Option<String>,
    active: bool,
    quota_per_day: Option<i32>,
}
```

## Success Criteria
- Server starts with DATABASE_URL → reads accounts/proxies/keys from DB
- Server starts without DATABASE_URL → falls back to JSON files
- mark_used/mark_failure persist to DB
- API key auth validates against DB
- Usage logged per request
