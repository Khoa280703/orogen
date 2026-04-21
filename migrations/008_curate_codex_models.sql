-- Keep only the Codex models confirmed working with the current upstream account type.

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

UPDATE models
SET active = false
WHERE provider_id = (SELECT id FROM providers WHERE slug = 'codex')
  AND slug NOT IN ('gpt-5.4', 'gpt-5.4-mini', 'gpt-5.3-codex', 'gpt-5.2');

UPDATE public_model_routes r
SET active = m.active
FROM public_models pm
JOIN models m ON m.slug = pm.slug
WHERE r.public_model_id = pm.id
  AND r.provider_slug = 'codex'
  AND m.provider_id = (SELECT id FROM providers WHERE slug = 'codex');

UPDATE public_models pm
SET
    display_name = m.name,
    description = m.description,
    active = m.active
FROM models m
WHERE pm.slug = m.slug
  AND m.provider_id = (SELECT id FROM providers WHERE slug = 'codex');
