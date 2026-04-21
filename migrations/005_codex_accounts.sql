-- Migration 005: Provider-aware accounts with Codex OAuth support

ALTER TABLE accounts ADD COLUMN IF NOT EXISTS provider_slug TEXT NOT NULL DEFAULT 'grok';
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS account_label TEXT;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS external_account_id TEXT;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS auth_mode TEXT NOT NULL DEFAULT 'grok_cookies';
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS metadata JSONB NOT NULL DEFAULT '{}'::jsonb;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS is_default BOOLEAN NOT NULL DEFAULT false;

CREATE TABLE IF NOT EXISTS account_credentials (
    account_id INTEGER PRIMARY KEY REFERENCES accounts(id) ON DELETE CASCADE,
    credential_type TEXT NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_accounts_provider_active
    ON accounts(provider_slug, active, is_default DESC, created_at ASC);

CREATE INDEX IF NOT EXISTS idx_account_credentials_type
    ON account_credentials(credential_type);

UPDATE accounts
SET
    provider_slug = COALESCE(NULLIF(provider_slug, ''), 'grok'),
    auth_mode = CASE
        WHEN COALESCE(auth_mode, '') = '' THEN 'grok_cookies'
        ELSE auth_mode
    END;

INSERT INTO account_credentials (account_id, credential_type, payload)
SELECT a.id, 'grok_cookies', a.cookies
FROM accounts a
WHERE a.provider_slug = 'grok'
  AND a.cookies IS NOT NULL
  AND NOT EXISTS (
      SELECT 1
      FROM account_credentials ac
      WHERE ac.account_id = a.id
  );

INSERT INTO providers (name, slug, active)
VALUES ('Codex', 'codex', true)
ON CONFLICT (slug) DO NOTHING;

INSERT INTO models (provider_id, name, slug, description, active, sort_order)
SELECT p.id, 'gpt-5.4', 'gpt-5.4', 'Confirmed working general model through the current Codex account.', true, 101
FROM providers p
WHERE p.slug = 'codex'
ON CONFLICT (slug) DO UPDATE
SET name = EXCLUDED.name, description = EXCLUDED.description, active = true, sort_order = EXCLUDED.sort_order;

INSERT INTO models (provider_id, name, slug, description, active, sort_order)
SELECT p.id, 'gpt-5.4-mini', 'gpt-5.4-mini', 'Confirmed working lighter Codex-routed model for faster requests.', true, 102
FROM providers p
WHERE p.slug = 'codex'
ON CONFLICT (slug) DO UPDATE
SET name = EXCLUDED.name, description = EXCLUDED.description, active = true, sort_order = EXCLUDED.sort_order;

INSERT INTO models (provider_id, name, slug, description, active, sort_order)
SELECT p.id, 'gpt-5.3-codex', 'gpt-5.3-codex', 'Confirmed working coding-focused model through the current Codex account.', true, 103
FROM providers p
WHERE p.slug = 'codex'
ON CONFLICT (slug) DO UPDATE
SET name = EXCLUDED.name, description = EXCLUDED.description, active = true, sort_order = EXCLUDED.sort_order;

INSERT INTO models (provider_id, name, slug, description, active, sort_order)
SELECT p.id, 'gpt-5.2', 'gpt-5.2', 'Confirmed working fallback general model through the current Codex account.', true, 104
FROM providers p
WHERE p.slug = 'codex'
ON CONFLICT (slug) DO UPDATE
SET name = EXCLUDED.name, description = EXCLUDED.description, active = true, sort_order = EXCLUDED.sort_order;
