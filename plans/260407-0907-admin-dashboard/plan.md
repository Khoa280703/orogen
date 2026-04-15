---
status: complete
---

# Admin Dashboard — Grok API Service

## Context
Chuyển từ JSON-file config sang PostgreSQL + Next.js admin UI.
Backend Rust đã có: account pool, proxy pool, API server, health monitoring.

## Phases

### Phase 1: PostgreSQL + Docker Setup ← `phase-01-postgresql-setup.md`
- Status: complete
- Docker Compose cho PostgreSQL
- Schema migrations (sqlx-cli)
- Thêm sqlx dependency vào Rust

### Phase 2: Rust DB Layer ← `phase-02-rust-db-layer.md`
- Status: complete  
- Database models + CRUD queries
- Migrate AccountPool, ProxyPool, ApiKeys từ JSON → PostgreSQL
- Giữ backward compat: fallback JSON nếu DB không có

### Phase 3: Rust Admin API ← `phase-03-rust-admin-api.md`
- Status: complete
- CRUD endpoints: /admin/proxies, /admin/accounts, /admin/api-keys
- Admin auth (simple token hoặc basic auth)
- Usage stats endpoint

### Phase 4: Next.js Project Setup ← `phase-04-nextjs-setup.md`
- Status: complete
- Init Next.js + shadcn/ui + TanStack Query
- Admin login page
- Layout + sidebar navigation

### Phase 5: Dashboard Pages ← `phase-05-dashboard-pages.md`
- Status: complete
- Dashboard overview (stats, charts)
- Proxy management (CRUD + status)
- Account management (CRUD + health)
- API key management (create, revoke, quota)
- Usage logs viewer

## Key Decisions
- sqlx (not Diesel/SeaORM) — lightweight, compile-time checked, async
- shadcn/ui — modern, customizable, not opinionated
- Single Docker Compose for PostgreSQL
- Admin auth: simple Bearer token (same as apiToken) for Phase 3, upgrade later
