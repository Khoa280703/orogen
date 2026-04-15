# Phase 4: Next.js Project Setup

## Overview
- Priority: Medium
- Status: complete
- Init Next.js admin dashboard project

## Requirements
- Next.js 15 + App Router
- shadcn/ui components
- TanStack Query cho data fetching
- Admin login (simple token-based, stored in cookie/localStorage)
- Sidebar layout

## Project Structure
```
web/
├── app/
│   ├── layout.tsx          — root layout with sidebar
│   ├── page.tsx            — redirect to /dashboard
│   ├── login/page.tsx      — admin login
│   ├── dashboard/page.tsx  — overview stats
│   ├── proxies/page.tsx    — proxy management
│   ├── accounts/page.tsx   — account management
│   ├── api-keys/page.tsx   — API key management
│   └── usage/page.tsx      — usage logs
├── components/
│   ├── sidebar.tsx
│   ├── data-table.tsx      — reusable table with shadcn
│   └── stats-card.tsx
├── lib/
│   ├── api.ts              — fetch wrapper with admin token
│   └── auth.ts             — token storage + middleware
├── package.json
├── next.config.ts
└── tailwind.config.ts
```

## Implementation Steps
1. `npx create-next-app@latest web` — TypeScript, Tailwind, App Router
2. `npx shadcn@latest init` — setup shadcn/ui
3. Add components: Button, Input, Table, Card, Dialog, Badge, Sidebar
4. Create `lib/api.ts` — wrapper fetch with Bearer admin token + base URL
5. Create login page — input admin token, save to localStorage
6. Create sidebar layout — Dashboard, Proxies, Accounts, API Keys, Usage
7. Setup TanStack Query provider
8. Add `next.config.ts` proxy rewrite to Rust backend (avoid CORS in prod)

## Dependencies
- next, react, react-dom
- @tanstack/react-query
- shadcn/ui (via CLI)
- lucide-react (icons)

## Success Criteria
- `npm run dev` starts on port 3000
- Login page saves admin token
- Sidebar navigation works
- API calls to Rust backend succeed
