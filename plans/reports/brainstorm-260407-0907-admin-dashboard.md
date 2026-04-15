# Brainstorm: Admin Dashboard cho Grok API Service

## Problem
Cần UI quản lý proxy, Grok accounts, API keys, usage stats. Hiện tại mọi thứ nằm trong JSON files, quản lý bằng tay.

## Agreed Solution
- **Frontend**: Next.js + shadcn/ui
- **Backend**: Extend Rust backend với admin API endpoints
- **Database**: PostgreSQL (Docker) thay thế JSON files
- **Scope**: Admin-only dashboard, chưa customer-facing

## Architecture
```
Next.js (3000) → Rust Admin API (5169/admin/*) → PostgreSQL
                  Rust Grok Proxy (5169/v1/*)   → PostgreSQL
```

## Database Schema (draft)
- `accounts`: name, cookies (jsonb), active, proxy_id, request_count, fail_count, created_at
- `proxies`: url, active, label, assigned_accounts, created_at
- `api_keys`: key, label, quota_per_day, usage_today, active, created_at
- `usage_logs`: api_key_id, account_id, model, status, latency_ms, tokens, created_at

## Tech Stack
- Rust: sqlx (async PostgreSQL driver) + migrations
- Next.js: shadcn/ui, TanStack Query, NextAuth (admin login)
- Docker: PostgreSQL container
- ORM-like: sqlx with compile-time checked queries

## Phases
1. PostgreSQL setup (Docker) + schema migrations
2. Rust admin API (CRUD proxies, accounts, API keys)
3. Migrate Rust backend from JSON files → PostgreSQL
4. Next.js project setup + admin auth
5. Dashboard pages (proxies, accounts, API keys, usage stats)

## Risks
- Migration from JSON → PostgreSQL needs careful handling
- Two processes to maintain (Rust + Next.js)
- sqlx compile-time query checking needs running DB during build

## Next Steps
- Tạo plan chi tiết theo phases
