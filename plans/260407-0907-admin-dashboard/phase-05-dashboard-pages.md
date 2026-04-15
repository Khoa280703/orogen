# Phase 5: Dashboard Pages

## Overview
- Priority: Medium
- Status: complete
- Implement tất cả admin pages với CRUD operations

## Pages

### 1. Dashboard Overview (`/dashboard`)
- Stats cards: total accounts, active accounts, total requests today, error rate
- Chart: requests per day (7 ngày gần nhất)
- Recent activity list

### 2. Proxy Management (`/proxies`)
- Table: url (masked password), label, active, assigned accounts count
- Actions: Add, Edit, Toggle active, Delete
- Dialog form: url, label
- Bulk import from text (format: ip:port:user:pass per line)

### 3. Account Management (`/accounts`)
- Table: name, active, proxy, request count, fail count, success count, last used
- Actions: Add, Edit cookies, Toggle active, Assign proxy, Delete
- Dialog form: name, cookies (JSON editor or paste raw), proxy select
- Health badge: green (healthy), yellow (1-2 fails), red (paused)

### 4. API Key Management (`/api-keys`)
- Table: label, key (masked), active, quota, usage today
- Actions: Create, Edit label/quota, Revoke
- Create dialog: label, quota_per_day (optional)
- Copy key button (full key shown only on create)

### 5. Usage Logs (`/usage`)
- Table: timestamp, api key, account, model, status, latency
- Filters: date range, api key, status
- Pagination

## Shared Components
- `DataTable` — sortable, filterable table with shadcn
- `StatsCard` — number + label + trend indicator
- `ConfirmDialog` — delete/revoke confirmation
- `StatusBadge` — active/inactive/error badges

## Implementation Steps
1. Dashboard overview page + stats cards
2. Proxy management page + CRUD dialogs
3. Account management page + CRUD dialogs
4. API key management page + create/revoke
5. Usage logs page + filters + pagination
6. Polish: loading states, error handling, toast notifications

## Success Criteria
- All CRUD operations work end-to-end (UI → Rust API → PostgreSQL)
- Tables paginated and sortable
- Forms validate input before submit
- Toast feedback on success/error
