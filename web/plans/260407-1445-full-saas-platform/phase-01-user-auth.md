# Phase 1: User Auth + DB Schema

## Overview
- Priority: Critical (blocker for all user-facing features)
- Status: pending

## Auth Strategy: Google OAuth (primary)
- Google OAuth 2.0 via NextAuth.js (Auth.js)
- No email/password needed initially → no email service, no verify, no forgot password
- Can add email/password later if needed

## DB Schema

**Table: users**
```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    name TEXT,
    avatar_url TEXT,
    provider TEXT DEFAULT 'google',  -- google, email (future)
    provider_id TEXT,                 -- Google sub ID
    locale TEXT DEFAULT 'en',
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

**Table: plans**
```sql
CREATE TABLE plans (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    requests_per_day INTEGER,
    requests_per_month INTEGER,
    price_usd NUMERIC(10,2),
    price_vnd INTEGER,
    features JSONB,
    active BOOLEAN DEFAULT true,
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

**Table: user_plans**
```sql
CREATE TABLE user_plans (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id),
    plan_id INTEGER NOT NULL REFERENCES plans(id),
    starts_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    active BOOLEAN DEFAULT true
);
```

**Table: balances**
```sql
CREATE TABLE balances (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL UNIQUE REFERENCES users(id),
    amount NUMERIC(10,2) DEFAULT 0,
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

**Modify existing tables**
```sql
ALTER TABLE api_keys ADD COLUMN user_id INTEGER REFERENCES users(id);
ALTER TABLE usage_logs ADD COLUMN user_id INTEGER REFERENCES users(id);
```

## Rust Backend

### New files
- `src/db/users.rs` — find_or_create_by_google, get_user
- `src/db/plans.rs` — list_plans, get_plan
- `src/db/user_plans.rs` — assign_plan, get_active_plan
- `src/db/balances.rs` — get_balance, add_credit, deduct
- `src/api/user_auth.rs` — POST /auth/google (verify Google token → create/find user → return JWT)

### Auth flow
```
Next.js (NextAuth) → Google OAuth → get Google token
Next.js → POST /auth/google { google_token } → Rust backend
Rust → verify token with Google API → find_or_create user → return JWT
User stores JWT → use for all /user/* endpoints
```

### Cargo.toml
```toml
jsonwebtoken = "9"
```

## Next.js

### Auth setup
- NextAuth.js (Auth.js) with Google provider
- On login success: call Rust /auth/google → get app JWT
- Store JWT in cookie/context

### New files
- `src/app/(auth)/login/page.tsx` — "Login with Google" button
- `src/lib/auth-context.tsx` — JWT state, user info
- `src/components/auth-guard.tsx` — protect user routes

### Route groups
- `(auth)` — login page
- `(admin)` — admin pages (existing admin token auth)
- `(user)` — user dashboard (JWT auth)
- `(public)` — landing, pricing, docs

### Google OAuth Setup
- Google Cloud Console → create OAuth 2.0 client
- Env: GOOGLE_CLIENT_ID, GOOGLE_CLIENT_SECRET, NEXTAUTH_SECRET

## Implementation Steps
1. Create migration `002_users_plans.sql`
2. Setup NextAuth.js with Google provider
3. Create Rust /auth/google endpoint
4. Add JWT middleware for /user/* routes
5. Create auth context + guard in Next.js
6. Seed default plans (Free, Pro, Enterprise)
7. Auto-assign Free plan on first login

## Success Criteria
- User clicks "Login with Google" → redirected → logged in
- JWT issued and stored
- User routes protected
- Free plan auto-assigned on first login
