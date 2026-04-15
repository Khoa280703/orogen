---
status: pending
created: 2026-04-08
slug: multi-provider-model-system
---

# Multi-Provider Model System + Plan Enforcement

## Context

duanai backend (Rust/Axum/PostgreSQL) currently has hardcoded model list and NO enforcement of plan limits or model access. Need to redesign for:
- Multiple providers (Grok now, ChatGPT later)
- Multiple models per provider
- Flexible plans granting access to specific models/providers
- Usage quota enforcement per plan

## Confirmed Models (tested working)
`grok-3`, `grok-3-thinking`, `grok-4`, `grok-4-auto`, `grok-4-thinking`

## Decisions
- Anonymous API keys (no user_id): **skip enforcement**
- Default plan for new users: **user decides later** — plans managed via admin CRUD
- Alias mapping (sonnet/opus/haiku): **removed**
- `features` JSONB on plans: **kept** for display, NOT for enforcement

## Phases

| Phase | File | Status |
|-------|------|--------|
| Phase 1 | [phase-01-db-schema-migration.md](phase-01-db-schema-migration.md) | Complete |
| Phase 2 | [phase-02-db-modules.md](phase-02-db-modules.md) | Complete |
| Phase 3 | [phase-03-plan-enforcement.md](phase-03-plan-enforcement.md) | Complete |
| Phase 4 | [phase-04-api-updates.md](phase-04-api-updates.md) | Complete |
| Phase 5 | [phase-05-admin-crud.md](phase-05-admin-crud.md) | Complete |

## Architecture

```
Request Flow:
1. API key auth (existing)
2. Resolve model slug (exact match from DB, no aliases)
3. IF user_id exists:
   a. Get active plan → plan_id
   b. Check model in plan_models → 403 if not allowed
   c. Check usage count vs plan limits → 429 if exceeded
4. Route to provider (Grok client)
```

## New DB Schema

```
providers:   id, name, slug (UNIQUE), active, created_at
models:      id, provider_id (FK), name, slug (UNIQUE), active, sort_order, created_at
plan_models: plan_id (FK), model_id (FK) — composite PK
```

## Verification
1. `cargo build` — compiles
2. DB: `SELECT * FROM providers/models/plan_models`
3. `GET /v1/models` — returns from DB
4. Anonymous key + any model — works (no enforcement)
5. User key on Free plan + grok-3 — works
6. User key on Free plan + grok-4 — 403 ModelNotAllowed
7. Admin CRUD for providers/models/plan-models
