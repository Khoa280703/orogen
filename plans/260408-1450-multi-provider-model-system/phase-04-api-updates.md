---
phase: 4
status: complete
priority: medium
completed: 2026-04-08
---

# Phase 4: API Updates

## Overview
Update model listing endpoint to be DB-driven. Remove hardcoded model mapping and aliases.

## Files
- **Update**: `src/api/models.rs` — dynamic model list from DB
- **Update**: `src/api/model_mapping.rs` — remove ADVERTISED_MODELS, remove aliases
- **Update**: `src/api/chat_completions.rs` — remove resolve_model_alias call

## `src/api/models.rs` Changes

Before: returns hardcoded ADVERTISED_MODELS list
After: queries `models` table from DB

```rust
// If authenticated user → filter by plan's allowed models
// If anonymous → return all active models
// Response format unchanged (OpenAI-compatible)
```

## `src/api/model_mapping.rs` Changes

Remove:
- `ADVERTISED_MODELS` constant
- `advertised_models_csv()` function
- `resolve_model_alias()` function (no more aliases)

Keep file minimal or delete entirely if no longer needed.

## `src/api/chat_completions.rs` Changes

Remove: call to `resolve_model_alias()`
The model slug from request is used directly (exact match against DB).
Validation happens in `enforce_plan_access()` (Phase 3).

## Success Criteria
- `GET /v1/models` returns models from DB
- `GET /v1/models` with user API key returns only plan-allowed models
- No hardcoded model list anywhere in code
- Invalid model slug → clear error message
