# Phase 4: Payment System

## Overview
- Priority: High
- Status: pending

## Payment Methods

### 1. Manual Topup (VN)
- User submits topup request: amount + transfer proof (screenshot or txn ID)
- Admin sees pending requests in admin panel
- Admin approves → balance credited
- Status: pending → approved/rejected

### 2. Crypto via fpayment (International)
- User selects amount → redirect to fpayment checkout
- fpayment sends webhook on payment confirmed
- Auto-credit balance
- Supported: USDT, BTC, ETH, TON

## DB Schema

**Table: transactions**
```sql
CREATE TABLE transactions (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id),
    type TEXT NOT NULL,            -- topup, deduction, refund
    method TEXT,                   -- manual, crypto
    amount NUMERIC(10,2) NOT NULL, -- USD
    currency TEXT DEFAULT 'USD',
    status TEXT DEFAULT 'pending', -- pending, completed, rejected
    reference TEXT,                -- bank txn ID, crypto txn hash
    proof_url TEXT,                -- screenshot URL for manual
    notes TEXT,                    -- admin notes
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_transactions_user ON transactions(user_id);
CREATE INDEX idx_transactions_status ON transactions(status);
```

## User Flow

### Manual Topup
```
User: /dashboard/billing → "Topup" → select amount → upload proof → submit
Admin: /admin/payments → see pending → approve/reject
System: balance += amount (on approve)
```

### Crypto Topup
```
User: /dashboard/billing → "Topup" → select amount → "Pay with Crypto"
System: create fpayment invoice → redirect user
fpayment: user pays → webhook POST /webhook/fpayment
System: verify signature → credit balance → mark completed
```

## Rust Backend

### New endpoints
- `POST /user/topup/manual` — submit manual request `{ amount, reference, proof_url }`
- `POST /user/topup/crypto` — create fpayment invoice → return checkout URL
- `POST /webhook/fpayment` — webhook receiver (no auth, verify signature)
- `GET /user/transactions` — list user's transactions

### Admin endpoints
- `GET /admin/payments` — list pending manual topups
- `PUT /admin/payments/:id/approve` — approve + credit balance
- `PUT /admin/payments/:id/reject` — reject with notes

### Balance deduction
- On each API request: deduct from balance based on plan pricing
- Or: plan-based (monthly subscription deducts fixed amount)
- **Recommend**: Credit-based (simpler) — user buys credits, each request costs X credits

## Files to Create
- `src/db/transactions.rs`
- `src/api/user_billing.rs` — topup endpoints
- `src/api/webhook_fpayment.rs` — crypto webhook
- `src/api/admin_payments.rs` — payment approval

### Next.js
- `src/app/(user)/dashboard/billing/topup/page.tsx` — topup flow
- `src/app/(admin)/payments/page.tsx` — payment approval queue

## fpayment Integration
- API: create invoice with amount, currency, callback URL
- Webhook: verify HMAC signature, extract txn details
- Env: FPAYMENT_API_KEY, FPAYMENT_SECRET

## Implementation Steps
1. Create migration `003_transactions.sql`
2. Implement transaction DB queries
3. Manual topup: user submit + admin approve flow
4. Crypto topup: fpayment invoice creation + webhook
5. Balance deduction logic on API requests
6. Next.js topup pages + admin payment queue
7. Test end-to-end: topup → balance → API usage → deduction

## Success Criteria
- Manual topup: user submits, admin approves, balance updates
- Crypto topup: user pays, webhook fires, balance auto-credits
- Balance deducted per API request
- Transaction history accurate
