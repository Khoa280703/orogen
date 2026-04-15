# DuanAI Project - Database Schema & Systems Exploration Report

**Date:** 2026-04-08  
**Report ID:** Explore-260408-duanai-db-schema

---

## 1. Complete Database Schema

### 1.1 Users Table
```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    name TEXT,
    avatar_url TEXT,
    provider TEXT DEFAULT 'google',          -- google, email (future)
    provider_id TEXT,                        -- Google sub ID
    locale TEXT DEFAULT 'en',
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_provider ON users(provider, provider_id);
```

**Fields:**
- `id`: Primary key
- `email`: Unique email address
- `name`: User's full name
- `avatar_url`: Profile avatar URL
- `provider`: Authentication provider (google, email)
- `provider_id`: OAuth provider ID (e.g., Google sub)
- `locale`: Language preference (default: en)
- `active`: Account status
- `created_at`: Account creation timestamp

---

### 1.2 Plans Table (Subscription Plans)
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
CREATE INDEX idx_plans_slug ON plans(slug);
CREATE INDEX idx_plans_active ON plans(active);
```

**Fields:**
- `id`: Primary key
- `name`: Plan display name
- `slug`: Unique URL-friendly identifier
- `requests_per_day`: Daily request quota (-1 = unlimited)
- `requests_per_month`: Monthly request quota (-1 = unlimited)
- `price_usd`: Price in USD
- `price_vnd`: Price in Vietnamese Dong
- `features`: JSONB containing plan features and restrictions
- `active`: Whether plan is available
- `sort_order`: Display order in UI

**Default Plans (seeded):**
1. **Free** (slug: free)
   - 10 requests/day, 300/month
   - Models: grok-3
   - Rate limit: 10/min
   - Price: $0

2. **Pro** (slug: pro)
   - 1000 requests/day, 30000/month
   - Models: grok-3, grok-4
   - Rate limit: 100/min
   - Priority support
   - Price: $29.99

3. **Enterprise** (slug: enterprise)
   - Unlimited requests
   - Models: grok-3, grok-4, grok-4-heavy
   - Rate limit: unlimited
   - Priority + dedicated support
   - Price: $199.99

---

### 1.3 User Plans Table (Subscriptions)
```sql
CREATE TABLE user_plans (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    plan_id INTEGER NOT NULL REFERENCES plans(id) ON DELETE RESTRICT,
    starts_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_user_plans_user ON user_plans(user_id);
CREATE INDEX idx_user_plans_active ON user_plans(active);
```

**Fields:**
- `id`: Primary key
- `user_id`: Reference to users table
- `plan_id`: Reference to plans table
- `starts_at`: Subscription start date
- `expires_at`: Subscription expiration date (NULL = no expiration)
- `active`: Whether this subscription is currently active
- `created_at`: Record creation timestamp

**Key Behavior:**
- When assigning a new plan to a user, existing active plans are deactivated
- A user can have multiple plan history records but only one active at a time
- Queries use `WHERE user_id = ? AND active = true ORDER BY starts_at DESC LIMIT 1` to get active plan

---

### 1.4 Balances Table (Credit System)
```sql
CREATE TABLE balances (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    amount NUMERIC(10,2) DEFAULT 0,
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_balances_user ON balances(user_id);
```

**Fields:**
- `id`: Primary key
- `user_id`: Unique reference to users (1:1 relationship)
- `amount`: Credit balance in USD (NUMERIC for precision)
- `updated_at`: Last update timestamp

**Operations:**
- `get_or_create_balance(user_id)`: Get or create new balance record
- `add_credit(user_id, amount)`: Add credit to user balance
- `deduct_credit(user_id, amount)`: Deduct only if sufficient balance exists

---

### 1.5 API Keys Table
```sql
CREATE TABLE api_keys (
    id SERIAL PRIMARY KEY,
    key TEXT NOT NULL UNIQUE,
    label TEXT,
    active BOOLEAN DEFAULT true,
    quota_per_day INTEGER,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    user_id INTEGER REFERENCES users(id) ON DELETE SET NULL,         -- Added in migration 002
    last_used_at TIMESTAMPTZ                                         -- Not in initial schema but used in code
);
CREATE INDEX idx_api_keys_active ON api_keys(active);
CREATE INDEX idx_api_keys_user ON api_keys(user_id);
```

**Fields:**
- `id`: Primary key
- `key`: Unique API key (format: sk-{base64})
- `label`: Human-readable name
- `active`: Whether key is usable
- `quota_per_day`: Daily request limit (NULL = no limit)
- `created_at`: Creation timestamp
- `user_id`: Reference to users table (can be NULL for legacy keys)
- `last_used_at`: Last usage timestamp (populated by code, possibly missing column)

**Key Generation:** `sk-{base64(16 random bytes)}`

---

### 1.6 Usage Logs Table
```sql
CREATE TABLE usage_logs (
    id BIGSERIAL PRIMARY KEY,
    api_key_id INTEGER REFERENCES api_keys(id) ON DELETE SET NULL,
    account_id INTEGER REFERENCES accounts(id) ON DELETE SET NULL,
    model TEXT,
    status TEXT,
    latency_ms INTEGER,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    user_id INTEGER REFERENCES users(id) ON DELETE SET NULL          -- Added in migration 002
);
CREATE INDEX idx_usage_logs_created ON usage_logs(created_at);
CREATE INDEX idx_usage_logs_api_key ON usage_logs(api_key_id);
CREATE INDEX idx_usage_logs_account ON usage_logs(api_key_id);
CREATE INDEX idx_usage_logs_user ON usage_logs(user_id);
```

**Fields:**
- `id`: Primary key (BIGSERIAL for high volume)
- `api_key_id`: Reference to api_keys
- `account_id`: Reference to accounts (proxy accounts)
- `user_id`: Reference to users (denormalized for query efficiency)
- `model`: Model name used (e.g., grok-3, grok-4)
- `status`: Request status (success, rate_limited, unauthorized, cf_blocked, error, retry_success)
- `latency_ms`: Response latency in milliseconds
- `created_at`: Request timestamp

---

### 1.7 Accounts Table (Proxy Accounts)
```sql
CREATE TABLE accounts (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    cookies JSONB NOT NULL,
    active BOOLEAN DEFAULT true,
    proxy_id INTEGER REFERENCES proxies(id) ON DELETE SET NULL,
    request_count BIGINT DEFAULT 0,
    fail_count INTEGER DEFAULT 0,
    success_count BIGINT DEFAULT 0,
    last_used TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_accounts_active ON accounts(active);
```

**Fields:**
- `id`: Primary key
- `name`: Account identifier
- `cookies`: JSONB storing session cookies
- `active`: Whether account is usable
- `proxy_id`: Optional reference to proxies table
- `request_count`: Total requests made
- `fail_count`: Failed requests
- `success_count`: Successful requests
- `last_used`: Last usage timestamp
- `created_at`: Account creation timestamp

---

### 1.8 Proxies Table
```sql
CREATE TABLE proxies (
    id SERIAL PRIMARY KEY,
    url TEXT NOT NULL UNIQUE,
    label TEXT,
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_proxies_active ON proxies(active);
```

**Fields:**
- `id`: Primary key
- `url`: Proxy URL (unique)
- `label`: Human-readable label
- `active`: Whether proxy is usable
- `created_at`: Creation timestamp

---

## 2. Plans System Architecture

### 2.1 Plan Structure & Features (JSONB)
Plans store flexible features in JSONB format. Seed example:
```json
{
  "models": ["grok-3"],
  "streaming": true,
  "rate_limit": "10/min"
}
```

**Stored in `plans.features` JSONB column.**

### 2.2 User-Plan Association
- Each user has **one active plan** at any time
- Historical records preserved (all previous plans stored with active=false)
- Plans can expire via `expires_at` timestamp (NULL = no expiration)
- When assigning new plan, system:
  1. Sets all existing active plans to `active = false`
  2. Creates new user_plans record with `active = true`

### 2.3 Plan Queries

**Get user's active plan:**
```sql
SELECT id, user_id, plan_id, starts_at, expires_at, active, created_at
FROM user_plans
WHERE user_id = $1 AND active = true
ORDER BY starts_at DESC
LIMIT 1
```

**List all active plans (for public display):**
```sql
SELECT id, name, slug, requests_per_day, requests_per_month,
       price_usd::text, price_vnd, features, active, sort_order, created_at
FROM plans
WHERE active = true
ORDER BY sort_order ASC
```

### 2.4 Plan Fields Used in Chat Completions

**Current Status:** Plan data is loaded but NOT enforced during request processing.

Seeded plans show:
- `requests_per_day`: Daily quota (not checked in code)
- `requests_per_month`: Monthly quota (not checked in code)
- `features.models`: Allowed models (NOT enforced - all advertised models usable)
- `features.rate_limit`: Rate limit spec (NOT enforced)
- `features.streaming`: Whether streaming allowed (NOT enforced)
- `features.priority`: Priority support indicator

---

## 3. API Key System

### 3.1 API Key Creation & Format
- **Format:** `sk-{base64(16 random bytes)}`
- **Generation:** Cryptographically secure random bytes encoded in base64url
- **Storage:** Plain text in database (column: `api_keys.key`)
- **Uniqueness:** `UNIQUE` constraint on key column

### 3.2 API Key Lifecycle

**Creation:**
```rust
pub async fn create_key(
    pool: &sqlx::PgPool,
    label: &str,
    user_id: Option<i32>,
) -> Result<ApiKey, sqlx::Error>
```
- Generates random key
- Stores with label and optional user_id
- Returns full key only on creation

**Validation:**
```rust
pub async fn get_key_by_value(
    pool: &sqlx::PgPool,
    key: &str,
) -> Result<Option<ApiKey>, sqlx::Error>
```
- Looks up key in database
- Returns ApiKey struct with user_id if found
- Checks `active = true` condition

**Revocation:**
```rust
pub async fn revoke_key(pool: &sqlx::PgPool, id: i32) -> Result<bool, sqlx::Error>
```
- Sets `active = false`
- Permanently disables key without deletion

### 3.3 API Key Fields & Tracking

**Stored Fields:**
- `id`: Primary key
- `key`: Unique key value
- `label`: Display name
- `user_id`: Associated user (NULL for legacy/admin keys)
- `active`: Active status
- `created_at`: Creation timestamp
- `quota_per_day`: Daily quota (stored but not enforced)
- `last_used_at`: Last usage timestamp

**Last Used Tracking:**
- Updated on each successful request via `touch_last_used()`
- Field may be missing in older db instances (added dynamically or in migrations)

### 3.4 API Key Usage in Chat Completions

**Resolution Flow:**
1. Client sends request with `Authorization: Bearer sk-...` header
2. Middleware extracts key into `ApiKey` extension
3. In `resolve_usage_context()`:
   - If key is empty: no tracking, returns None for api_key_id and user_id
   - If key provided: looks up in database via `get_key_by_value()`
   - Returns `UsageContext { api_key_id, user_id, model }`
4. Usage is logged with api_key_id and user_id

**Quota Checking:**
- `quota_per_day` field stored but **NOT enforced** in code
- No usage checking against limits during request processing

---

## 4. Model Resolution & Chat Completions Handler

### 4.1 Advertised Models

```rust
pub const ADVERTISED_MODELS: &[&str] = &[
    "grok-3",
    "grok-4",
    "grok-latest",
    "grok-3-thinking",
    "sonnet",
    "opus",
    "haiku",
];
```

### 4.2 Model Alias Resolution

**Function:** `resolve_model_alias(requested_model: &str) -> (String, bool)`

**Resolution Rules:**
1. If model contains "thinking" or "reasoning" → maps to `("grok-3", is_thinking=true)`
2. If model is "sonnet", "opus", or "haiku" → maps to `("grok-4", is_thinking=false)`
3. If model is "grok-3", "grok-4", or "grok-latest" → returns as-is with `is_thinking=false`
4. If model starts with "claude-" → maps to `("grok-4", is_thinking=false)`
5. Otherwise → returns normalized model name as-is

**Output:**
- `(resolved_model_name, is_thinking_flag)`
- `is_thinking` is used to construct GrokRequest payload differently

### 4.3 Chat Completions Request Flow

```
Client Request
    ↓
Extract API Key from header → ApiKey extension
    ↓
Resolve usage context (api_key_id, user_id)
    ↓
Resolve model alias (requested_model → effective_model, is_thinking)
    ↓
Flatten messages (system + user/assistant)
    ↓
Create GrokRequest payload
    ↓
[If streaming]
  → Return SSE stream with chat.completion.chunk events
[If non-streaming]
  → Send request, parse response, return full completion
    ↓
Log usage (api_key_id, user_id, account_id, status, latency_ms)
    ↓
Update api_key.last_used_at (if api_key_id present)
```

### 4.4 Request Routing & Account Management

**Account Selection:**
- `state.pool.get_current()` returns currently active account
- On rate limit or unauthorized: rotates to next account via `state.pool.rotate()`
- One retry attempt with rotated account

**Status Tracking in Logs:**
- `success`: Request completed
- `rate_limited`: Hit rate limit, may retry with another account
- `retry_success`: Retry after rotation succeeded
- `unauthorized`: Account cookies expired/invalid, may rotate
- `cf_blocked`: Cloudflare blocking (daemon handles CF bypass)
- `error`: Other errors

---

## 5. Plan-to-Model Restrictions

### 5.1 Current State

**Status:** NO restrictions enforced.

Evidence:
1. `resolve_usage_context()` retrieves api_key_id and user_id only - no plan lookup
2. No code in chat_completions that validates model against user's plan features
3. All advertised models are equally accessible regardless of plan
4. Default model fallback used if no model specified: `state.config.default_model`

### 5.2 Plan Features Not Enforced

Despite plans storing these features in JSONB:
- `features.models`: Array of allowed models (e.g., `["grok-3", "grok-4"]`)
- `features.rate_limit`: Rate limit specification (e.g., `"10/min"`)
- `features.streaming`: Whether streaming is allowed
- `features.priority`: Support priority indicator

**None of these are checked during request processing.**

### 5.3 Quota System Not Enforced

- `api_keys.quota_per_day`: Stored but never checked
- `plans.requests_per_day`: Stored but never checked
- `plans.requests_per_month`: Stored but never checked

---

## 6. Key Insights & Findings

### 6.1 Database Design Strengths
- ✅ Proper foreign key constraints with cascading deletes
- ✅ Appropriate indexes for common queries (user_id, api_key, created_at)
- ✅ JSONB for flexible plan features
- ✅ Separate balance tracking for credit system
- ✅ Comprehensive usage logging with user + api_key tracking

### 6.2 Current Gaps / TODO Items
- ❌ Plan feature enforcement not implemented
- ❌ Daily/monthly quota checking not implemented
- ❌ Model-to-plan validation not implemented
- ❌ Rate limiting not enforced per plan specs
- ❌ `last_used_at` field may be missing from older migrations
- ❌ No payment/subscription expiration checking
- ❌ No usage tracking in real-time billing system

### 6.3 Audit Trail
- ✅ All user actions logged in usage_logs with timestamps
- ✅ API key tracking enables user-level analytics
- ✅ Account rotation tracked (retry_success status)
- ✅ Latency metrics captured per request

---

## 7. Data Flow Example

### Example: User Making Chat Completion Request

```
1. User (user_id=5) calls /v1/chat/completions
   - API Key: sk-abc123def456...
   - Model: "sonnet"
   - Message: "Hello"

2. Middleware extracts key → ApiKey("sk-abc123def456...")

3. resolve_usage_context():
   - Queries: SELECT * FROM api_keys WHERE key = 'sk-abc123def456...' AND active = true
   - Finds: ApiKey { id=10, user_id=5, ... }
   - Returns: UsageContext { api_key_id=10, user_id=5, model="sonnet" }

4. resolve_model_alias("sonnet"):
   - Returns: ("grok-4", false)

5. Get active plan:
   - Queries: SELECT * FROM user_plans WHERE user_id=5 AND active=true LIMIT 1
   - Finds: UserPlan { plan_id=2, ... }
   - [Plan features NOT checked]

6. Send request to Grok API via account pool
   - Account used: accounts[current_index]
   - Proxy: proxies[account.proxy_id]

7. On success:
   - Log usage:
     INSERT INTO usage_logs (api_key_id=10, user_id=5, account_id=3, model='sonnet', status='success', latency_ms=245)
   - Update last_used:
     UPDATE api_keys SET last_used_at=NOW() WHERE id=10

8. Return response with chat.completion format
```

---

## 8. Missing/Uncertain Details

### Potential Issues
1. **`api_keys.last_used_at`**: Code uses this field but may not exist in initial schema
   - Migration 002 doesn't explicitly add it
   - Likely added via separate migration or dynamically

2. **User-plan lifecycle**: When plan expires via `expires_at`:
   - No automatic downgrade to free plan
   - No explicit query checking expiration during requests
   - System assumes active=true in user_plans persists

3. **Balance updates**: Unclear when/how credits are deducted:
   - No usage tracking tied to credits
   - No payment processing visible in explored code

4. **Account rotation logic**: `state.pool` implementation not explored
   - Likely rounds-robin or weighted selection
   - Rate limit detection and rotation strategy unknown

---

## Files Explored

- `/home/khoa2807/working-sources/duanai/migrations/001_initial_schema.sql`
- `/home/khoa2807/working-sources/duanai/migrations/002_users_plans.sql`
- `/home/khoa2807/working-sources/duanai/src/db/plans.rs`
- `/home/khoa2807/working-sources/duanai/src/db/api_keys.rs`
- `/home/khoa2807/working-sources/duanai/src/db/user_plans.rs`
- `/home/khoa2807/working-sources/duanai/src/db/usage_logs.rs`
- `/home/khoa2807/working-sources/duanai/src/db/balances.rs`
- `/home/khoa2807/working-sources/duanai/src/db/users.rs`
- `/home/khoa2807/working-sources/duanai/src/api/model_mapping.rs`
- `/home/khoa2807/working-sources/duanai/src/api/chat_completions.rs`
- `/home/khoa2807/working-sources/duanai/src/api/admin_plans.rs`
- `/home/khoa2807/working-sources/duanai/src/api/admin_api_keys.rs`

