# Phase 6: Frontend — User Dashboard Rewrite

## Context
- [Phase 4 — Chat UI](./phase-04-frontend-chat-ui.md)
- [Phase 5 — Image Studio](./phase-05-frontend-image-studio.md)
- Current dashboard: keys, usage, billing, settings pages
- Goal: rewrite as media-studio-oriented hub

## Overview
- **Priority**: P2
- **Status**: Complete
- **Effort**: 5h

Redesign user dashboard as studio hub. Keep billing/settings. Replace usage stats with studio activity overview.

## Key Insights
- Current dashboard pages (keys, usage, billing, settings) mostly work — refactor not rewrite
- Main dashboard page needs redesign: show recent chats, recent images, quick actions
- Sidebar updates: Chat + Images as primary nav, dashboard/billing/settings secondary
- Public pages (landing, pricing, docs) need minor updates for new positioning

## Related Code Files

| File | Action | Purpose |
|------|--------|---------|
| `web/src/app/(app)/dashboard/page.tsx` | REWRITE | Studio hub: recent activity + quick actions |
| `web/src/app/(app)/dashboard/keys/page.tsx` | KEEP | Minor text updates |
| `web/src/app/(app)/dashboard/usage/page.tsx` | MODIFY | Add chat/image usage breakdown |
| `web/src/app/(app)/dashboard/billing/page.tsx` | KEEP | Works as-is |
| `web/src/app/(app)/dashboard/settings/page.tsx` | KEEP | Works as-is |
| `web/src/components/user-sidebar.tsx` | MODIFY | Reorder nav: Chat, Images first |
| `web/src/app/(app)/layout.tsx` | MODIFY | Update layout if needed |
| `web/src/app/(public)/page.tsx` | MODIFY | Update hero for media studio positioning |
| `web/src/app/(public)/pricing/page.tsx` | MODIFY | Update feature descriptions |

## Implementation Steps

1. **Rewrite dashboard/page.tsx** (~150 lines):
   - Quick actions: "New Chat", "Generate Image" cards
   - Recent conversations (last 5) with links
   - Recent image generations (last 5) with thumbnails
   - Usage summary: credits remaining, requests today
   - Plan info: current plan, upgrade CTA

2. **Update user-sidebar.tsx** (~70 lines):
   - Reorder navigation:
     - Chat (primary)
     - Images (primary)
     - Dashboard (secondary)
     - Usage (secondary)
     - Billing (secondary)
     - API Keys (secondary)
     - Settings (secondary)
   - Group with dividers

3. **Update usage page** — add breakdown by type (chat vs image)

4. **Update public landing page** — hero text/description for media studio

5. **Update pricing page** — feature list reflects chat + image gen

## Todo List
- [x] Rewrite dashboard/page.tsx
- [x] Update user-sidebar.tsx nav order
- [x] Update usage page
- [x] Update landing page hero
- [x] Update pricing page features
- [x] `npx tsc --noEmit`
- [x] `npm run build` to verify
- [x] Browser smoke test `/dashboard`, `/dashboard/usage`, `/pricing`
- [x] Browser smoke test authenticated `/chat` and `/images` entry points after nav changes

## Success Criteria
- Dashboard shows recent activity (chats + images)
- Quick action cards navigate to chat/images
- Sidebar prioritizes studio features
- Landing page positions as media studio
- All existing pages still work

## Current Progress
- Dashboard overview rewritten into a studio hub using existing profile, usage, conversation, and image history APIs
- Usage page now shows request volume with chat/image creation breakdown
- Sidebar grouped into `Create` and `Manage` sections with Chat/Images prioritized
- Landing and pricing pages now position the product as a media studio instead of API-proxy-first
- `npx tsc --noEmit` and `npm run build` pass in `web/`
- Public pages and authenticated dashboard entry points were smoke tested successfully with a JWT-backed session against the latest backend build

## Risk Assessment
- **Low**: mostly UI reorganization, no complex logic
- **Residual risk**: usage page derives chat/image counters on the client, so very large accounts may need a dedicated aggregated backend endpoint later
- **Residual risk**: local Grok upstream credentials are currently unauthorized, so dashboard-linked chat/image actions can still surface external service errors even though app routing and consumer API plumbing are working
