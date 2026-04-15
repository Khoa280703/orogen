---
phase: 1
status: complete
priority: high
completed: 2026-04-08
---

# Phase 1: DB Schema Migration

## Overview
Create `providers`, `models`, `plan_models` tables. Seed Grok provider + 5 models + plan-model associations.

## Files
- **Create**: `migrations/003_providers_models.sql`
- **Update**: `src/db/migrate.rs` — add migration 003

## Migration SQL

```sql
CREATE TABLE IF NOT EXISTS providers (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS models (
    id SERIAL PRIMARY KEY,
    provider_id INTEGER NOT NULL REFERENCES providers(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    active BOOLEAN DEFAULT true,
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS plan_models (
    plan_id INTEGER NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    model_id INTEGER NOT NULL REFERENCES models(id) ON DELETE CASCADE,
    PRIMARY KEY (plan_id, model_id)
);

CREATE INDEX IF NOT EXISTS idx_models_provider ON models(provider_id);
CREATE INDEX IF NOT EXISTS idx_models_slug ON models(slug);
CREATE INDEX IF NOT EXISTS idx_plan_models_plan ON plan_models(plan_id);
CREATE INDEX IF NOT EXISTS idx_plan_models_model ON plan_models(model_id);
```

## Seed Data

Provider: `grok`

Models:
| slug | name | sort_order |
|------|------|-----------|
| grok-3 | Grok 3 | 1 |
| grok-3-thinking | Grok 3 Thinking | 2 |
| grok-4 | Grok 4 | 3 |
| grok-4-auto | Grok 4 Auto | 4 |
| grok-4-thinking | Grok 4 Thinking | 5 |

Plan-model associations (based on existing plans):
- Free → grok-3
- Pro → grok-3, grok-3-thinking, grok-4, grok-4-auto, grok-4-thinking
- Enterprise → all models

## Success Criteria
- Migration runs without errors
- `SELECT * FROM providers/models/plan_models` returns correct data
