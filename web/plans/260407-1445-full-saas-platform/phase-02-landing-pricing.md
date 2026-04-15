# Phase 2: Landing Page + Pricing

## Overview
- Priority: High
- Status: pending

## Pages

### Landing Page (`/`)
- Hero: tagline + CTA "Get Started"
- Features grid: fast, cheap, OpenAI-compatible, streaming
- How it works: 3 steps (register → get key → call API)
- Testimonials/stats (optional, placeholder initially)
- Footer: links, socials

### Pricing Page (`/pricing`)
- Fetch plans from API: GET /api/plans
- 3 cards: Free / Pro / Enterprise
- Feature comparison table
- CTA buttons → register or upgrade

### i18n
- next-intl or simple context-based switching
- Toggle vi/en in header
- Content: landing, pricing, auth pages, dashboard labels
- Approach: JSON locale files `messages/vi.json`, `messages/en.json`

## Files to Create
- `src/app/(public)/layout.tsx` — public layout (header + footer, no sidebar)
- `src/app/(public)/page.tsx` — landing page
- `src/app/(public)/pricing/page.tsx` — pricing page
- `src/components/public-header.tsx` — nav with login/register + locale toggle
- `src/components/public-footer.tsx`
- `src/components/pricing-card.tsx`
- `messages/vi.json`, `messages/en.json`

## Rust Backend
- `GET /api/plans` — public endpoint, list active plans with pricing

## Implementation Steps
1. Create public layout (header, footer)
2. Build landing page with hero + features
3. Build pricing page with dynamic plan cards
4. Add locale switching (vi/en)
5. Add /api/plans endpoint to Rust
6. Responsive design (mobile-first)

## Success Criteria
- Landing page loads, looks professional
- Pricing shows plans from DB
- Locale toggle switches vi ↔ en
- All pages responsive
