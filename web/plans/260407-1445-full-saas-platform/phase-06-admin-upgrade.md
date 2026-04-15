# Phase 6: Admin Upgrade

## Overview
- Priority: Medium
- Status: complete
- Completed: 2026-04-07

## New Admin Pages

### User Management (`/admin/users`)
- Table: email, name, plan, balance, active, registered date
- Actions: view details, adjust balance, change plan, ban/unban
- Search + filter by plan, status

### Payment Approval (`/admin/payments`)
- Queue: pending manual topups
- Show: user, amount, reference, proof image
- Actions: approve (credits balance), reject (with notes)
- Filter by status: pending, completed, rejected

### Plan Management (`/admin/plans`)
- CRUD plans: name, slug, price, limits, features
- Toggle active/inactive
- Sort order for pricing page

### Revenue Dashboard (`/admin/revenue`)
- Total revenue (today, week, month)
- Revenue chart (30 days)
- Revenue by payment method (manual vs crypto)
- Active subscribers count
- Top users by spend

### System Health (`/admin/health`)
- Grok account health (from existing /admin/accounts)
- Proxy status
- API request volume chart
- Error rate
- Active users (last 24h)

## Rust Backend Additions

### Admin endpoints
- `GET /admin/users` — list users + plan + balance
- `GET /admin/users/:id` — user detail + usage + transactions
- `PUT /admin/users/:id` — update active, plan, balance
- `GET /admin/plans` — list plans (existing, extend)
- `POST /admin/plans` — create plan
- `PUT /admin/plans/:id` — update plan
- `GET /admin/revenue` — aggregated revenue stats
- `GET /admin/health` — system health summary

### DB queries
- `src/db/users.rs` — list_users, get_user_detail, update_user
- `src/db/plans.rs` — CRUD (extend existing)
- `src/db/transactions.rs` — revenue aggregation queries

## Files to Create/Modify

### Next.js
- `src/app/(admin)/users/page.tsx`
- `src/app/(admin)/payments/page.tsx`
- `src/app/(admin)/plans/page.tsx`
- `src/app/(admin)/revenue/page.tsx`
- `src/app/(admin)/health/page.tsx`
- Update admin sidebar with new links

### Rust
- `src/api/admin_users.rs`
- `src/api/admin_plans.rs` (extend)
- `src/api/admin_revenue.rs`
- Extend existing admin modules

## Implementation Steps
1. User management page + backend
2. Payment approval queue (connect to Phase 4)
3. Plan CRUD management
4. Revenue dashboard with charts
5. System health dashboard
6. Update admin sidebar navigation

## Success Criteria
- Admin can manage all users (view, ban, adjust)
- Manual payments approved/rejected from admin
- Plans editable from UI
- Revenue stats accurate
- System health visible at a glance
