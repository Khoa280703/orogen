# Phase 1: PostgreSQL + Docker Setup

## Overview
- Priority: High (blocker cho tất cả phases sau)
- Status: complete

## Requirements
- Docker Compose file cho PostgreSQL 16
- Database schema với 4 tables: accounts, proxies, api_keys, usage_logs
- sqlx-cli cho migrations
- Rust dependencies: sqlx với postgres + runtime-tokio

## Implementation Steps

### 1. Docker Compose
Tạo `docker-compose.yml`:
- PostgreSQL 16 alpine
- Port 5432
- Volume persist data
- Environment: POSTGRES_DB=grok_local, POSTGRES_USER, POSTGRES_PASSWORD

### 2. Cargo.toml Dependencies
```toml
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "chrono", "uuid", "json"] }
```

### 3. Database Schema (migrations)

**Table: proxies**
```sql
CREATE TABLE proxies (
    id SERIAL PRIMARY KEY,
    url TEXT NOT NULL UNIQUE,
    label TEXT,
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

**Table: accounts**
```sql
CREATE TABLE accounts (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    cookies JSONB NOT NULL,
    active BOOLEAN DEFAULT true,
    proxy_id INTEGER REFERENCES proxies(id),
    request_count BIGINT DEFAULT 0,
    fail_count INTEGER DEFAULT 0,
    success_count BIGINT DEFAULT 0,
    last_used TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

**Table: api_keys**
```sql
CREATE TABLE api_keys (
    id SERIAL PRIMARY KEY,
    key TEXT NOT NULL UNIQUE,
    label TEXT,
    active BOOLEAN DEFAULT true,
    quota_per_day INTEGER,  -- NULL = unlimited
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

**Table: usage_logs**
```sql
CREATE TABLE usage_logs (
    id BIGSERIAL PRIMARY KEY,
    api_key_id INTEGER REFERENCES api_keys(id),
    account_id INTEGER REFERENCES accounts(id),
    model TEXT,
    status TEXT,  -- success, rate_limited, cf_blocked, error
    latency_ms INTEGER,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_usage_logs_created ON usage_logs(created_at);
CREATE INDEX idx_usage_logs_api_key ON usage_logs(api_key_id);
```

### 4. Environment Config
Tạo `.env`:
```
DATABASE_URL=postgres://grok:grokpass@localhost:5432/grok_local
```

### 5. Rust Config Update
- Thêm `database_url: Option<String>` vào AppConfig
- Load từ env var DATABASE_URL hoặc config.json

## Files to Create
- `docker-compose.yml`
- `migrations/001_initial_schema.sql`
- `.env` (git-ignored)

## Files to Modify
- `Cargo.toml` — add sqlx
- `src/config.rs` — add database_url
- `.gitignore` — add .env

## Success Criteria
- `docker compose up -d` starts PostgreSQL
- `sqlx migrate run` creates all tables
- `cargo build` compiles with sqlx
