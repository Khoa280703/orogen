---
phase: 5
status: complete
priority: medium
completed: 2026-04-08
---

# Phase 5: Admin CRUD

## Overview
Admin endpoints to manage providers, models, and plan-model associations.

## Files
- **Create**: `src/api/admin_providers.rs` (~80 lines)
- **Create**: `src/api/admin_models.rs` (~100 lines)
- **Update**: `src/api/admin_plans.rs` — add plan-model management endpoints
- **Update**: `src/api/mod.rs` — register new routes

## Admin Endpoints

### Providers
```
GET    /admin/providers           → list all providers
POST   /admin/providers           → create provider { name, slug }
PUT    /admin/providers/:id       → update provider { name, active }
```

### Models
```
GET    /admin/models              → list all models (with provider info)
POST   /admin/models              → create model { provider_id, name, slug, sort_order }
PUT    /admin/models/:id          → update model { name, active, sort_order }
DELETE /admin/models/:id          → deactivate model (soft delete)
```

### Plan-Model Associations
```
GET    /admin/plans/:id/models    → list models in plan
POST   /admin/plans/:id/models    → add model(s) to plan { model_ids: [1,2,3] }
DELETE /admin/plans/:id/models/:model_id → remove model from plan
PUT    /admin/plans/:id/models    → replace all models { model_ids: [1,2,3] }
```

## Success Criteria
- All CRUD operations work via curl
- Creating new provider + models + assigning to plan works end-to-end
- Future: ChatGPT provider can be added via admin API without code changes
