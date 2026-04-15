---
phase: 2
status: complete
priority: high
completed: 2026-04-08
---

# Phase 2: DB Modules

## Overview
Rust CRUD modules for providers, models, plan_models tables.

## Files
- **Create**: `src/db/providers.rs` (~50 lines)
- **Create**: `src/db/models.rs` (~70 lines)
- **Create**: `src/db/plan_models.rs` (~40 lines)
- **Update**: `src/db/mod.rs` — register new modules

## `src/db/providers.rs`

```rust
struct Provider { id, name, slug, active, created_at }

// Functions:
list_providers(pool) -> Vec<Provider>
get_provider(pool, id) -> Option<Provider>
create_provider(pool, name, slug) -> Provider
update_provider(pool, id, name, active) -> Option<Provider>
```

## `src/db/models.rs`

```rust
struct Model { id, provider_id, name, slug, active, sort_order, created_at }

// Functions:
list_models(pool) -> Vec<Model>                           // all active models
list_models_by_provider(pool, provider_id) -> Vec<Model>
get_model_by_slug(pool, slug) -> Option<Model>
list_models_for_plan(pool, plan_id) -> Vec<Model>         // JOIN plan_models
is_model_allowed_for_plan(pool, plan_id, model_slug) -> bool
create_model(pool, provider_id, name, slug) -> Model
update_model(pool, id, name, active, sort_order) -> Option<Model>
```

## `src/db/plan_models.rs`

```rust
// Functions:
add_model_to_plan(pool, plan_id, model_id) -> Result
remove_model_from_plan(pool, plan_id, model_id) -> Result
list_plan_models(pool, plan_id) -> Vec<Model>             // JOIN models
set_plan_models(pool, plan_id, model_ids: Vec<i32>) -> Result  // replace all
```

## Success Criteria
- `cargo build` compiles
- All functions work with test queries
