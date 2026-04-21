CREATE TABLE IF NOT EXISTS plan_public_models (
    plan_id INTEGER NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    public_model_id INTEGER NOT NULL REFERENCES public_models(id) ON DELETE CASCADE,
    PRIMARY KEY (plan_id, public_model_id)
);

CREATE INDEX IF NOT EXISTS idx_plan_public_models_plan
    ON plan_public_models(plan_id);

CREATE INDEX IF NOT EXISTS idx_plan_public_models_public_model
    ON plan_public_models(public_model_id);

INSERT INTO plan_public_models (plan_id, public_model_id)
SELECT DISTINCT plm.plan_id, pm.id
FROM plan_models plm
JOIN models m ON m.id = plm.model_id
JOIN public_models pm ON pm.slug = m.slug
ON CONFLICT (plan_id, public_model_id) DO NOTHING;
