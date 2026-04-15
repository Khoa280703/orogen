-- Migration 003: Providers, Models, Plan-Model associations
-- Created: 2026-04-08
-- Purpose: Multi-provider model system with plan enforcement

-- Providers table
CREATE TABLE IF NOT EXISTS providers (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Models table
CREATE TABLE IF NOT EXISTS models (
    id SERIAL PRIMARY KEY,
    provider_id INTEGER NOT NULL REFERENCES providers(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    description TEXT,
    active BOOLEAN DEFAULT true,
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Plan-Model associations (junction table)
CREATE TABLE IF NOT EXISTS plan_models (
    plan_id INTEGER NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    model_id INTEGER NOT NULL REFERENCES models(id) ON DELETE CASCADE,
    PRIMARY KEY (plan_id, model_id)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_models_provider ON models(provider_id);
CREATE INDEX IF NOT EXISTS idx_models_slug ON models(slug);
CREATE INDEX IF NOT EXISTS idx_plan_models_plan ON plan_models(plan_id);
CREATE INDEX IF NOT EXISTS idx_plan_models_model ON plan_models(model_id);

-- Seed Grok provider
INSERT INTO providers (name, slug, active)
VALUES ('Grok', 'grok', true)
ON CONFLICT (slug) DO NOTHING;

-- Seed Grok models
INSERT INTO models (provider_id, name, slug, description, active, sort_order)
SELECT p.id, 'Grok 3', 'grok-3', 'Balanced default model for fast everyday chat.', true, 1 FROM providers p WHERE p.slug = 'grok'
ON CONFLICT (slug) DO NOTHING;

INSERT INTO models (provider_id, name, slug, description, active, sort_order)
SELECT p.id, 'Grok 3 Thinking', 'grok-3-thinking', 'Extra reasoning depth for harder prompts.', true, 2 FROM providers p WHERE p.slug = 'grok'
ON CONFLICT (slug) DO NOTHING;

INSERT INTO models (provider_id, name, slug, description, active, sort_order)
SELECT p.id, 'Grok 4', 'grok-4', 'Higher quality answers for demanding tasks.', true, 3 FROM providers p WHERE p.slug = 'grok'
ON CONFLICT (slug) DO NOTHING;

INSERT INTO models (provider_id, name, slug, description, active, sort_order)
SELECT p.id, 'Grok 4 Auto', 'grok-4-auto', 'Auto-tuned Grok 4 mode for mixed workloads.', true, 4 FROM providers p WHERE p.slug = 'grok'
ON CONFLICT (slug) DO NOTHING;

INSERT INTO models (provider_id, name, slug, description, active, sort_order)
SELECT p.id, 'Grok 4 Thinking', 'grok-4-thinking', 'Deep reasoning variant for complex problem solving.', true, 5 FROM providers p WHERE p.slug = 'grok'
ON CONFLICT (slug) DO NOTHING;

-- Seed plan-model associations
-- Free plan: grok-3 only
INSERT INTO plan_models (plan_id, model_id)
SELECT p.id, m.id FROM plans p CROSS JOIN models m WHERE p.slug = 'free' AND m.slug = 'grok-3'
ON CONFLICT (plan_id, model_id) DO NOTHING;

-- Pro plan: all 5 models
INSERT INTO plan_models (plan_id, model_id)
SELECT p.id, m.id FROM plans p CROSS JOIN models m WHERE p.slug = 'pro' AND m.slug IN ('grok-3', 'grok-3-thinking', 'grok-4', 'grok-4-auto', 'grok-4-thinking')
ON CONFLICT (plan_id, model_id) DO NOTHING;

-- Enterprise plan: all 5 models
INSERT INTO plan_models (plan_id, model_id)
SELECT p.id, m.id FROM plans p CROSS JOIN models m WHERE p.slug = 'enterprise' AND m.slug IN ('grok-3', 'grok-3-thinking', 'grok-4', 'grok-4-auto', 'grok-4-thinking')
ON CONFLICT (plan_id, model_id) DO NOTHING;
