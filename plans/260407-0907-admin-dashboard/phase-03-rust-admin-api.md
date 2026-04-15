# Phase 3: Rust Admin API

## Overview
- Priority: High
- Status: complete
- CRUD endpoints cho admin UI consume

## Requirements
- RESTful JSON API tại /admin/*
- Admin auth: Bearer token (reuse apiToken hoặc dedicated admin_token)
- CRUD: proxies, accounts, api_keys
- Stats: usage summary, account health

## Endpoints

### Proxies
- `GET /admin/proxies` — list all
- `POST /admin/proxies` — create `{ url, label }`
- `PUT /admin/proxies/:id` — update `{ url?, label?, active? }`
- `DELETE /admin/proxies/:id` — delete (fail if accounts assigned)

### Accounts
- `GET /admin/accounts` — list all with proxy info
- `POST /admin/accounts` — create `{ name, cookies, proxy_id? }`
- `PUT /admin/accounts/:id` — update `{ cookies?, active?, proxy_id? }`
- `DELETE /admin/accounts/:id` — delete

### API Keys
- `GET /admin/api-keys` — list all (masked keys)
- `POST /admin/api-keys` — create `{ label, quota_per_day? }` → returns full key
- `PUT /admin/api-keys/:id` — update `{ label?, active?, quota_per_day? }`
- `DELETE /admin/api-keys/:id` — revoke

### Stats
- `GET /admin/stats/overview` — total accounts, active, requests today, errors today
- `GET /admin/stats/usage?days=7` — daily usage breakdown

## Files to Create
- `src/api/admin_proxies.rs`
- `src/api/admin_accounts.rs`
- `src/api/admin_api_keys.rs`
- `src/api/admin_stats.rs`

## Files to Modify
- `src/api/mod.rs` — add admin routes, admin auth middleware
- `src/config.rs` — add `admin_token: Option<String>`

## Implementation Steps
1. Add admin_token to config
2. Create admin auth middleware (separate from API key auth)
3. Implement proxy CRUD handlers
4. Implement account CRUD handlers
5. Implement api_key CRUD handlers (generate random key on create)
6. Implement stats handlers with SQL aggregation queries
7. Wire all routes in api/mod.rs under /admin prefix
8. Add CORS for Next.js dev server (localhost:3000)

## Success Criteria
- All CRUD endpoints return proper JSON
- Admin token required for all /admin/* routes
- Create API key returns full key (only time it's visible)
- Stats endpoint returns aggregated usage data
