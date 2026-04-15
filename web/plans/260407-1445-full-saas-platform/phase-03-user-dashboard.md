# Phase 3: User Dashboard

## Overview
- Priority: High
- Status: pending

## Pages

### Dashboard Overview (`/dashboard`)
- Current plan + usage (requests today / limit)
- Usage chart (7 days)
- Quick actions: create key, view docs

### API Keys (`/dashboard/keys`)
- List user's keys (masked)
- Create new key (label, show once)
- Revoke key
- Copy button

### Usage (`/dashboard/usage`)
- Table: timestamp, key, model, status, latency
- Filters: date range, key, model
- Daily/monthly aggregation
- Export CSV (optional)

### Account Settings (`/dashboard/settings`)
- Update name, email, password
- Current plan info
- Upgrade plan button → pricing
- Locale preference

### Billing (`/dashboard/billing`)
- Current balance
- Transaction history (topups, deductions)
- Topup button → payment flow

## Rust Backend Endpoints (user-scoped, JWT auth)
- `GET /user/me` — profile + plan + balance
- `GET /user/keys` — list user's API keys
- `POST /user/keys` — create key (auto-assign to user)
- `DELETE /user/keys/:id` — revoke
- `GET /user/usage?days=7` — usage stats
- `GET /user/billing` — balance + transactions

## Files to Create
- `src/app/(user)/layout.tsx` — user sidebar layout
- `src/app/(user)/dashboard/page.tsx`
- `src/app/(user)/dashboard/keys/page.tsx`
- `src/app/(user)/dashboard/usage/page.tsx`
- `src/app/(user)/dashboard/settings/page.tsx`
- `src/app/(user)/dashboard/billing/page.tsx`
- `src/components/user-sidebar.tsx`
- `src/components/usage-chart.tsx`

## Rust Backend Files
- `src/api/user.rs` — all /user/* endpoints
- `src/middleware/jwt_auth.rs` — extract user from JWT

## Key Logic: Usage Limit Enforcement
- On each /v1/chat/completions request:
  1. Validate API key → get user_id
  2. Get user's active plan → requests_per_day limit
  3. Count today's usage for this user
  4. If over limit → 429 with "quota exceeded"
  5. Else → proceed + log usage

## Implementation Steps
1. Create user layout + sidebar
2. Build dashboard overview with stats
3. Build API keys page (CRUD)
4. Build usage page with table + chart
5. Build settings page
6. Build billing page (balance + history)
7. Add Rust /user/* endpoints
8. Add quota enforcement in chat_completions

## Success Criteria
- User sees their plan, usage, balance
- User can create/revoke API keys
- Usage stats accurate and filterable
- Quota enforcement blocks over-limit requests
