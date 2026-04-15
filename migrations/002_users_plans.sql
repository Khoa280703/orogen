-- Migration 002: Users, Plans, Balances for SaaS platform
-- Created: 2026-04-07

-- Users table
CREATE TABLE IF NOT EXISTS users (
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
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_provider ON users(provider, provider_id);

-- Plans table
CREATE TABLE IF NOT EXISTS plans (
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
CREATE INDEX IF NOT EXISTS idx_plans_slug ON plans(slug);
CREATE INDEX IF NOT EXISTS idx_plans_active ON plans(active);

-- User plans (subscriptions)
CREATE TABLE IF NOT EXISTS user_plans (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    plan_id INTEGER NOT NULL REFERENCES plans(id) ON DELETE RESTRICT,
    starts_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_user_plans_user ON user_plans(user_id);
CREATE INDEX IF NOT EXISTS idx_user_plans_active ON user_plans(active);

-- Balances (credit system)
CREATE TABLE IF NOT EXISTS balances (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    amount NUMERIC(10,2) DEFAULT 0,
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_balances_user ON balances(user_id);

-- Modify existing api_keys table to add user_id
ALTER TABLE api_keys ADD COLUMN IF NOT EXISTS user_id INTEGER REFERENCES users(id) ON DELETE SET NULL;
CREATE INDEX IF NOT EXISTS idx_api_keys_user ON api_keys(user_id);

-- Modify existing usage_logs table to add user_id
ALTER TABLE usage_logs ADD COLUMN IF NOT EXISTS user_id INTEGER REFERENCES users(id) ON DELETE SET NULL;
CREATE INDEX IF NOT EXISTS idx_usage_logs_user ON usage_logs(user_id);

-- Seed default plans
INSERT INTO plans (name, slug, requests_per_day, requests_per_month, price_usd, price_vnd, features, active, sort_order)
VALUES
    ('Free', 'free', -1, 300, 0, 0, '{"models": ["grok-3"], "streaming": true, "rate_limit": "10/min", "model_limits": {"grok-3": {"chat_per_day": 10, "video_per_day": 1}, "imagine-x-1": {"image_per_day": 2}}}'::jsonb, true, 0),
    ('Pro', 'pro', 1000, 30000, 29.99, 750000, '{"models": ["grok-3", "grok-4"], "streaming": true, "rate_limit": "100/min", "priority": true}'::jsonb, true, 1),
    ('Enterprise', 'enterprise', -1, -1, 199.99, 5000000, '{"models": ["grok-3", "grok-4", "grok-4-heavy"], "streaming": true, "rate_limit": "unlimited", "priority": true, "dedicated_support": true}'::jsonb, true, 2)
ON CONFLICT (slug) DO NOTHING;
