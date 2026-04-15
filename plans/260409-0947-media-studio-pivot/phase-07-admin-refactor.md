# Phase 7: Admin Panel Refactor

## Context
- Current admin: 11 pages (dashboard, users, plans, API keys, accounts, proxies, payments, revenue, health, usage, models/providers)
- Goal: minor refactor, no rewrite. Add conversation/image gen monitoring.

## Overview
- **Priority**: P3
- **Status**: Complete
- **Effort**: 3h

Add admin visibility into consumer activity (conversations, image generations). Minor cleanup.

## Implementation Steps

1. **New admin page: Conversations** (~150 lines):
   - `web/src/app/(admin)/admin/conversations/page.tsx`
   - List all conversations across users (admin view)
   - Search by user/title and filter by model
   - View conversation messages (read-only)
   - Delete conversations from admin view

2. **New admin page: Image Generations** (~120 lines):
   - `web/src/app/(admin)/admin/images/page.tsx`
   - List all image generations across users
   - Search by user/prompt and filter by status
   - View prompts and generated images
   - Delete inappropriate content records

3. **Backend admin endpoints** (~80 lines):
   - `src/api/admin_conversations.rs` — list_all, get_detail, delete
   - `src/api/admin_images.rs` — list_all, get_detail, delete
   - Register in `src/api/mod.rs`

4. **Update admin sidebar** — add Conversations + Images links

5. **Update admin dashboard** — add conversation/image gen counts to stats

## Related Code Files

| File | Action | Purpose |
|------|--------|---------|
| `src/api/admin_conversations.rs` | CREATE | Admin conversation endpoints |
| `src/api/admin_images.rs` | CREATE | Admin image gen endpoints |
| `src/api/mod.rs` | MODIFY | Register admin routes |
| `web/src/app/(admin)/admin/conversations/page.tsx` | CREATE | Conversations admin page |
| `web/src/app/(admin)/admin/images/page.tsx` | CREATE | Image gens admin page |
| `web/src/components/admin-sidebar.tsx` | MODIFY | Add nav links |
| `web/src/lib/api.ts` | MODIFY | Add admin API functions |

## Todo List
- [x] Create admin_conversations.rs
- [x] Create admin_images.rs
- [x] Update mod.rs routes
- [x] Create admin conversations page
- [x] Create admin images page
- [x] Update admin sidebar
- [x] Update api.ts
- [x] `cargo build && npm run build`

## Success Criteria
- Admin can view all conversations and image generations
- Search and model/status filtering work for admin review flows
- Can inspect detail and delete content
- Stats dashboard shows new metrics

## Current Progress
- Added backend admin endpoints in `src/api/admin_conversations.rs` and `src/api/admin_images.rs`, wired through `src/api/mod.rs`
- Added admin pages at `web/src/app/(admin)/admin/conversations/page.tsx` and `web/src/app/(admin)/admin/images/page.tsx`
- Added sidebar links and frontend client helpers in `web/src/components/admin-sidebar.tsx` and `web/src/lib/api.ts`
- Admin dashboard now shows total conversations and total image generations
- Conversation delete is soft-delete (`active = false`); image delete removes the generation row

## Verification
- 2026-04-09: `cargo build` passed at repo root
- 2026-04-09: `npm run build` passed in `web/`
- 2026-04-09: Next.js production build emitted `/admin/conversations` and `/admin/images`

## Blocker Status
- Phase 7: none
- Overall pivot: live generation still blocked by upstream Grok credentials returning `Unauthorized` or `invalid-credentials`
