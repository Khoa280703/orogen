---
phase: 3
status: complete
priority: high
completed: 2026-04-08
---

# Phase 3: Plan Enforcement

## Overview
Core enforcement logic: check model access + usage quota before routing requests.

## Files
- **Create**: `src/api/plan_enforcement.rs` (~100 lines)
- **Update**: `src/error.rs` — add error variants
- **Update**: `src/api/chat_completions.rs` — add enforcement call

## Error Variants (src/error.rs)

```rust
ModelNotAllowed,    // 403 — "Model not available in your plan"
QuotaExceeded,      // 429 — "Daily request limit exceeded"
PlanRequired,       // 403 — "Active plan required"
```

## Enforcement Logic (src/api/plan_enforcement.rs)

```rust
pub async fn enforce_plan_access(
    db: &PgPool,
    user_id: Option<i32>,
    model_slug: &str,
) -> Result<(), AppError> {
    // 1. No user_id (anonymous key) → skip enforcement
    let user_id = match user_id {
        Some(id) => id,
        None => return Ok(()),
    };

    // 2. Get active plan
    let plan = get_active_plan(db, user_id).await?
        .ok_or(AppError::PlanRequired)?;

    // 3. Check model allowed
    if !is_model_allowed_for_plan(db, plan.plan_id, model_slug).await? {
        return Err(AppError::ModelNotAllowed);
    }

    // 4. Check daily quota
    let today_count = count_today_by_user(db, user_id).await?;
    if plan.requests_per_day > 0 && today_count >= plan.requests_per_day as i64 {
        return Err(AppError::QuotaExceeded);
    }

    Ok(())
}
```

## Integration in chat_completions.rs

Insert AFTER `resolve_usage_context()`, BEFORE `send_with_retry()`:

```rust
// After line ~67 (resolve_usage_context)
enforce_plan_access(&state.db, usage_context.user_id, &model).await?;
```

## Success Criteria
- Anonymous key: no enforcement, all models work
- User with Free plan + grok-3: works
- User with Free plan + grok-4: returns 403
- User exceeded daily quota: returns 429
