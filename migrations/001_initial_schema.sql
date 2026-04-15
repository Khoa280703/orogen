-- Proxies table
CREATE TABLE proxies (
    id SERIAL PRIMARY KEY,
    url TEXT NOT NULL UNIQUE,
    label TEXT,
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Accounts table
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

-- API keys table
CREATE TABLE api_keys (
    id SERIAL PRIMARY KEY,
    key TEXT NOT NULL UNIQUE,
    label TEXT,
    active BOOLEAN DEFAULT true,
    quota_per_day INTEGER,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Usage logs table
CREATE TABLE usage_logs (
    id BIGSERIAL PRIMARY KEY,
    api_key_id INTEGER REFERENCES api_keys(id) ON DELETE SET NULL,
    account_id INTEGER REFERENCES accounts(id) ON DELETE SET NULL,
    model TEXT,
    status TEXT,
    latency_ms INTEGER,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_usage_logs_created ON usage_logs(created_at);
CREATE INDEX idx_usage_logs_api_key ON usage_logs(api_key_id);
CREATE INDEX idx_usage_logs_account ON usage_logs(account_id);
CREATE INDEX idx_accounts_active ON accounts(active);
CREATE INDEX idx_proxies_active ON proxies(active);
CREATE INDEX idx_api_keys_active ON api_keys(active);
