# Next.js Admin Plans Management Page - Exploration Report

## Overview
Explored the Next.js admin dashboard at `/home/khoa2807/working-sources/duanai/web` to understand the plans management page structure, form implementation, and API integration.

---

## 1. Admin Plans Page File Path
**Frontend:** `/home/khoa2807/working-sources/duanai/web/src/app/(admin)/admin/plans/page.tsx`

---

## 2. Page Structure & Features

### Architecture
- **Type:** Client-side component (`'use client'`)
- **Framework:** Next.js App Router with shadcn/ui components
- **State Management:** React hooks (useState, useEffect)
- **API Client:** Custom `adminFetch` wrapper (uses Bearer token auth)

### Main Features Implemented
1. **Plan List View** - Table showing all plans with sorting by `sort_order`
2. **Create Plan Dialog** - Modal form for creating new plans
3. **Edit Plan Dialog** - Modal form for editing existing plans
4. **Inline Actions** - Active/Inactive toggle for plans
5. **Plan Details Display** - Shows name, slug, pricing, request limits

---

## 3. Form Fields & Data Model

### Create Plan Form Fields
```typescript
interface CreatePlanFormState {
  name: string;              // e.g., "Pro"
  slug: string;              // e.g., "pro" (URL-friendly identifier)
  requests_per_day: string;  // Integer, -1 = unlimited
  requests_per_month: string;// Integer, -1 = unlimited
  price_usd: string;         // Decimal number
  price_vnd: string;         // Integer
  features: string;          // JSON string
  active: boolean;           // true/false
  sort_order: number;        // Order in plan list
}
```

### Edit Plan Form Fields
```typescript
interface EditPlanFormState {
  id: number | null;
  name: string;
  requests_per_day: string;
  requests_per_month: string;
  price_usd: string;
  price_vnd: string;
  features: string;          // JSON string
  active: boolean;
  sort_order: string;        // Note: string type (different from create)
}
```

### Features Field Handling (JSONB)
- **Input Type:** HTML `<Textarea>` component for JSON editing
- **Format:** JSON string that must be valid JSON
- **Default:** `"{}"`
- **Example Placeholder:** `{"models": ["grok-3"], "streaming": true}`
- **Processing:**
  - On create: `JSON.parse(newPlan.features)` converts string to object
  - On edit: `JSON.stringify(plan.features || {}, null, 2)` converts object to formatted JSON
  - Backend: Stored as JSONB in PostgreSQL

### Current Form UI Components
- **Input components:** For text and number fields
- **Textarea component:** For JSON features field (monospace font)
- **Badge component:** For active status (display only, not editable in list)
- **Dialog component:** Modal wrapper for forms
- **Button component:** For submit and toggle actions
- **Table component:** For plan list display

---

## 4. API Endpoints Called

### Frontend API Calls (via `adminFetch`)
The frontend calls these endpoints on the **backend** at `http://localhost:3069/admin/`:

| Method | Endpoint | Purpose |
|--------|----------|---------|
| GET | `/admin/plans` | List all plans (no active filter on admin side) |
| POST | `/admin/plans` | Create new plan |
| PUT | `/admin/plans/:id` | Update plan (partial update) |

**Note:** The frontend `route.ts` file only proxies requests to the backend, it doesn't implement plan logic.

---

## 5. Backend API Implementation

### Backend File
**Path:** `/home/khoa2807/working-sources/duanai/src/api/admin_plans.rs`

### API Routes Registered (in `/home/khoa2807/working-sources/duanai/src/api/mod.rs`)
```rust
// Plans
.route("/plans", get(admin_plans::list_plans))
.route("/plans", post(admin_plans::create_plan))
.route("/plans/:id", put(admin_plans::update_plan))
.route("/plans/:id/models", get(admin_plans::list_plan_models))
.route("/plans/:id/models", post(admin_plans::add_models_to_plan))
.route("/plans/:id/models", put(admin_plans::set_all_plan_models))
.route("/plans/:id/models/:model_id", delete(admin_plans::remove_model_from_plan_endpoint))
```

### Plan Response Schema
```rust
pub struct PlanResponse {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub requests_per_day: Option<i32>,
    pub requests_per_month: Option<i32>,
    pub price_usd: Option<String>,
    pub price_vnd: Option<i32>,
    pub features: Option<Value>,           // JSONB field
    pub active: bool,
    pub sort_order: i32,
    pub created_at: Option<String>,
}
```

### Create Plan Request Schema
```rust
pub struct PlanCreateRequest {
    pub name: String,
    pub slug: String,
    pub requests_per_day: Option<i32>,
    pub requests_per_month: Option<i32>,
    pub price_usd: Option<String>,
    pub price_vnd: Option<i32>,
    pub features: Option<Value>,           // Can be any serde_json::Value
    pub active: bool,
    pub sort_order: i32,
}
```

### Update Plan Request Schema
```rust
pub struct PlanUpdateRequest {
    pub name: Option<String>,
    pub requests_per_day: Option<i32>,
    pub requests_per_month: Option<i32>,
    pub price_usd: Option<String>,
    pub price_vnd: Option<i32>,
    pub features: Option<Value>,
    pub active: Option<bool>,
    pub sort_order: Option<i32>,
}
```

---

## 6. Plan-Model Associations

### Database Schema
- **Junction Table:** `plan_models` (plan_id, model_id)
- **Models Table:** Contains model definitions with provider_id and sort_order
- **Relationship:** Many-to-many association between plans and models

### Seeded Plans & Models
```sql
-- Free plan: grok-3 only
-- Pro plan: grok-3, grok-3-thinking, grok-4, grok-4-auto, grok-4-thinking
-- Enterprise plan: all 5 models
```

### Model Association Endpoints (NOT implemented in frontend yet)
- `GET /admin/plans/:id/models` - List models in plan
- `POST /admin/plans/:id/models` - Add model(s) to plan
- `PUT /admin/plans/:id/models` - Replace all models in plan
- `DELETE /admin/plans/:id/models/:model_id` - Remove model from plan

### Important: Model Selection Not in Frontend
**Current Status:** The admin plans page does NOT have UI for selecting/managing models for plans. This functionality:
- Exists in backend API
- Exists in database schema
- **Missing from frontend** - needs to be implemented if required

---

## 7. Database Schema (Plans Table)

```sql
CREATE TABLE IF NOT EXISTS plans (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    requests_per_day INTEGER,           -- -1 = unlimited
    requests_per_month INTEGER,         -- -1 = unlimited
    price_usd NUMERIC(10,2),
    price_vnd INTEGER,
    features JSONB,                     -- JSON field for features
    active BOOLEAN DEFAULT true,
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

---

## 8. Current Form Limitations & Issues

### Issues Found
1. **No Model Selection UI:** Backend supports plan-model associations, but frontend form doesn't allow selecting models
2. **Active Badge Display-Only:** In the edit dialog, the active status is shown as a badge but cannot be toggled (toggle only works from the table row)
3. **Update Logic Issue:** The PUT endpoint uses conditional if-else logic that only updates ONE field at a time - cannot update multiple fields in a single request
4. **Missing Validation:** Frontend doesn't validate JSON in features field before sending

### Missing Field in UI
- **Model Selection:** No UI component to select which models can be used with each plan
- **Plan-Model Management:** No separate page/section to manage plan-model associations

---

## 9. Data Type Conversions

### Frontend → Backend Conversions (Create)
```typescript
{
  ...newPlan,
  requests_per_day: newPlan.requests_per_day ? parseInt(...) : null,
  requests_per_month: newPlan.requests_per_month ? parseInt(...) : null,
  price_usd: newPlan.price_usd ? parseFloat(...) : null,
  price_vnd: newPlan.price_vnd ? parseInt(...) : null,
  features: newPlan.features ? JSON.parse(...) : {},
}
```

### Frontend → Backend Conversions (Edit)
```typescript
{
  name: editPlan.name,
  requests_per_day: editPlan.requests_per_day ? parseInt(...) : null,
  requests_per_month: editPlan.requests_per_month ? parseInt(...) : null,
  price_usd: editPlan.price_usd ? parseFloat(...) : null,
  price_vnd: editPlan.price_vnd ? parseInt(...) : null,
  features: editPlan.features ? JSON.parse(...) : null,
  active: editPlan.active,
  sort_order: editPlan.sort_order ? parseInt(...) : null,
}
```

---

## 10. Seeded Data

### Default Plans (from migration)
```json
[
  {
    "slug": "free",
    "name": "Free",
    "requests_per_day": 10,
    "requests_per_month": 300,
    "price_usd": 0,
    "price_vnd": 0,
    "features": {
      "models": ["grok-3"],
      "streaming": true,
      "rate_limit": "10/min"
    },
    "sort_order": 0
  },
  {
    "slug": "pro",
    "name": "Pro",
    "requests_per_day": 1000,
    "requests_per_month": 30000,
    "price_usd": 29.99,
    "price_vnd": 750000,
    "features": {
      "models": ["grok-3", "grok-4"],
      "streaming": true,
      "rate_limit": "100/min",
      "priority": true
    },
    "sort_order": 1
  },
  {
    "slug": "enterprise",
    "name": "Enterprise",
    "requests_per_day": -1,
    "requests_per_month": -1,
    "price_usd": 199.99,
    "price_vnd": 5000000,
    "features": {
      "models": ["grok-3", "grok-4", "grok-4-heavy"],
      "streaming": true,
      "rate_limit": "unlimited",
      "priority": true,
      "dedicated_support": true
    },
    "sort_order": 2
  }
]
```

---

## 11. Authentication & Authorization

- **Admin Token:** Bearer token stored in `localStorage` as `admin_token`
- **Header:** All admin requests include `Authorization: Bearer {token}`
- **CSRF Protection:** Automatic CSRF token refresh and inclusion for state-changing requests
- **Backend Middleware:** `/admin/*` routes protected by admin token validation

---

## 12. Key Files Summary

| Path | Purpose |
|------|---------|
| `/home/khoa2807/working-sources/duanai/web/src/app/(admin)/admin/plans/page.tsx` | Admin plans UI component |
| `/home/khoa2807/working-sources/duanai/src/api/admin_plans.rs` | Backend API handlers |
| `/home/khoa2807/working-sources/duanai/src/db/plans.rs` | Database queries for plans |
| `/home/khoa2807/working-sources/duanai/src/db/plan_models.rs` | Database queries for plan-model associations |
| `/home/khoa2807/working-sources/duanai/migrations/002_users_plans.sql` | Plans table schema |
| `/home/khoa2807/working-sources/duanai/migrations/003_providers_models.sql` | Models and plan_models tables |
| `/home/khoa2807/working-sources/duanai/web/src/lib/api.ts` | API client with auth |

---

## 13. Unresolved Questions

1. **Model Selection in UI:** Should model selection be added to the create/edit plan forms?
2. **Backend Update Logic:** Why use conditional if-else in PUT endpoint instead of dynamic field updates?
3. **JSON Validation:** Should the frontend validate JSON before sending?
4. **Active Toggle in Edit:** Should the edit form allow toggling active status?
5. **Default Values:** Should there be default model assignments when creating a plan?

