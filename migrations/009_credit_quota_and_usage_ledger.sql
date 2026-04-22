ALTER TABLE api_keys ADD COLUMN IF NOT EXISTS daily_credit_limit BIGINT;
ALTER TABLE api_keys ADD COLUMN IF NOT EXISTS monthly_credit_limit BIGINT;
ALTER TABLE api_keys ADD COLUMN IF NOT EXISTS max_input_tokens INTEGER;
ALTER TABLE api_keys ADD COLUMN IF NOT EXISTS max_output_tokens INTEGER;

ALTER TABLE usage_logs ADD COLUMN IF NOT EXISTS plan_id INTEGER REFERENCES plans(id) ON DELETE SET NULL;
ALTER TABLE usage_logs ADD COLUMN IF NOT EXISTS prompt_tokens BIGINT NOT NULL DEFAULT 0;
ALTER TABLE usage_logs ADD COLUMN IF NOT EXISTS completion_tokens BIGINT NOT NULL DEFAULT 0;
ALTER TABLE usage_logs ADD COLUMN IF NOT EXISTS cached_tokens BIGINT NOT NULL DEFAULT 0;
ALTER TABLE usage_logs ADD COLUMN IF NOT EXISTS credits_used BIGINT NOT NULL DEFAULT 0;
ALTER TABLE usage_logs ADD COLUMN IF NOT EXISTS estimated_usage BOOLEAN NOT NULL DEFAULT false;

CREATE INDEX IF NOT EXISTS idx_api_keys_daily_credit_limit
    ON api_keys(daily_credit_limit);

CREATE INDEX IF NOT EXISTS idx_usage_logs_plan_created
    ON usage_logs(plan_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_usage_logs_user_credits_created
    ON usage_logs(user_id, created_at DESC, credits_used);

CREATE INDEX IF NOT EXISTS idx_usage_logs_api_key_credits_created
    ON usage_logs(api_key_id, created_at DESC, credits_used);
