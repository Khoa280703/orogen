ALTER TABLE accounts ADD COLUMN IF NOT EXISTS routing_state TEXT NOT NULL DEFAULT 'candidate';
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS cooldown_until TIMESTAMPTZ;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS last_routing_error TEXT;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS rate_limit_streak INTEGER NOT NULL DEFAULT 0;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS auth_failure_streak INTEGER NOT NULL DEFAULT 0;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS refresh_failure_streak INTEGER NOT NULL DEFAULT 0;

CREATE INDEX IF NOT EXISTS idx_accounts_provider_routing_state
    ON accounts(provider_slug, active, routing_state, created_at ASC);

CREATE INDEX IF NOT EXISTS idx_accounts_cooldown_until
    ON accounts(cooldown_until);

UPDATE accounts
SET routing_state = CASE
    WHEN COALESCE(active, true) = false THEN 'paused'
    WHEN session_status = 'expired' THEN 'auth_invalid'
    WHEN session_status = 'refresh_failed' THEN 'refresh_failed'
    WHEN session_status = 'healthy' THEN 'healthy'
    ELSE COALESCE(NULLIF(routing_state, ''), 'candidate')
END;

CREATE TABLE IF NOT EXISTS public_models (
    id SERIAL PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    description TEXT,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS public_model_routes (
    id SERIAL PRIMARY KEY,
    public_model_id INTEGER NOT NULL REFERENCES public_models(id) ON DELETE CASCADE,
    provider_slug TEXT NOT NULL,
    upstream_model_slug TEXT NOT NULL,
    route_priority INTEGER NOT NULL DEFAULT 0,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(public_model_id, provider_slug, upstream_model_slug)
);

CREATE INDEX IF NOT EXISTS idx_public_models_active
    ON public_models(active, slug);

CREATE INDEX IF NOT EXISTS idx_public_model_routes_lookup
    ON public_model_routes(public_model_id, active, route_priority, provider_slug);

INSERT INTO public_models (slug, display_name, description, active, created_at)
SELECT m.slug, m.name, m.description, m.active, COALESCE(m.created_at, NOW())
FROM models m
ON CONFLICT (slug) DO UPDATE
SET
    display_name = EXCLUDED.display_name,
    description = EXCLUDED.description,
    active = EXCLUDED.active;

INSERT INTO public_model_routes (public_model_id, provider_slug, upstream_model_slug, route_priority, active)
SELECT pm.id, p.slug, m.slug, 0, m.active
FROM public_models pm
JOIN models m ON m.slug = pm.slug
JOIN providers p ON p.id = m.provider_id
ON CONFLICT (public_model_id, provider_slug, upstream_model_slug) DO UPDATE
SET
    route_priority = EXCLUDED.route_priority,
    active = EXCLUDED.active;
