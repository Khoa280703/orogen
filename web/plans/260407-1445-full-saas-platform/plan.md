---
status: in_progress
---

# Full SaaS API Platform

## Context
Nâng cấp Grok API proxy từ admin tool → commercial SaaS.
Existing: Rust backend (proxy, accounts, admin API) + Next.js admin dashboard + PostgreSQL.

## Phases

### Phase 1: User Auth + DB Schema ← `phase-01-user-auth.md`
- Status: complete
- User registration, login, JWT, email verification
- DB: users, plans, user_plans, balances

### Phase 2: Landing Page + Pricing ← `phase-02-landing-pricing.md`
- Status: complete
- Hero, features, pricing cards
- i18n (vi/en)

### Phase 3: User Dashboard ← `phase-03-user-dashboard.md`
- Status: complete
- API key management, usage stats, account settings

### Phase 4: Payment System ← `phase-04-payment-system.md`
- Status: complete
- Manual topup (VN), Crypto/fpayment (Intl)
- Balance/credit system

### Phase 5: API Docs + Guides ← `phase-05-api-docs.md`
- Status: complete
- MDX docs, API reference, code examples

### Phase 6: Admin Upgrade ← `phase-06-admin-upgrade.md`
- Status: complete
- User mgmt, payment approval, plan CRUD, revenue dashboard

### Phase 7: Production Hardening ← `phase-07-production-hardening.md`
- Status: complete
- Email service, forgot password, rate limiting, deployment (Docker+Nginx+SSL)
- SEO, error pages, Telegram alerts, legal pages, backup, logging
