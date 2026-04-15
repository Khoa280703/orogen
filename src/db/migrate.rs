use sqlx::postgres::PgPoolOptions;

/// Run database migrations
pub async fn run_migrations(database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    // Migration 002: Users table
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            email TEXT NOT NULL UNIQUE,
            name TEXT,
            avatar_url TEXT,
            provider TEXT DEFAULT 'google',
            provider_id TEXT,
            locale TEXT DEFAULT 'en',
            active BOOLEAN DEFAULT true,
            created_at TIMESTAMPTZ DEFAULT NOW()
        )"#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(r#"CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)"#)
        .execute(&pool)
        .await?;

    sqlx::query(r#"CREATE INDEX IF NOT EXISTS idx_users_provider ON users(provider, provider_id)"#)
        .execute(&pool)
        .await?;

    // Plans table
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS plans (
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
        )"#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(r#"CREATE INDEX IF NOT EXISTS idx_plans_slug ON plans(slug)"#)
        .execute(&pool)
        .await?;

    sqlx::query(r#"CREATE INDEX IF NOT EXISTS idx_plans_active ON plans(active)"#)
        .execute(&pool)
        .await?;

    // User plans table
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS user_plans (
            id SERIAL PRIMARY KEY,
            user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            plan_id INTEGER NOT NULL REFERENCES plans(id) ON DELETE RESTRICT,
            starts_at TIMESTAMPTZ DEFAULT NOW(),
            expires_at TIMESTAMPTZ,
            active BOOLEAN DEFAULT true,
            created_at TIMESTAMPTZ DEFAULT NOW()
        )"#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(r#"CREATE INDEX IF NOT EXISTS idx_user_plans_user ON user_plans(user_id)"#)
        .execute(&pool)
        .await?;

    sqlx::query(r#"CREATE INDEX IF NOT EXISTS idx_user_plans_active ON user_plans(active)"#)
        .execute(&pool)
        .await?;

    // Balances table
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS balances (
            id SERIAL PRIMARY KEY,
            user_id INTEGER NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
            amount NUMERIC(10,2) DEFAULT 0,
            updated_at TIMESTAMPTZ DEFAULT NOW()
        )"#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(r#"CREATE INDEX IF NOT EXISTS idx_balances_user ON balances(user_id)"#)
        .execute(&pool)
        .await?;

    // Modify api_keys table
    sqlx::query(
        r#"ALTER TABLE api_keys ADD COLUMN IF NOT EXISTS user_id INTEGER REFERENCES users(id) ON DELETE SET NULL"#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"ALTER TABLE api_keys ADD COLUMN IF NOT EXISTS plan_id INTEGER REFERENCES plans(id) ON DELETE SET NULL"#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(r#"ALTER TABLE api_keys ADD COLUMN IF NOT EXISTS last_used_at TIMESTAMPTZ"#)
        .execute(&pool)
        .await?;

    sqlx::query(r#"ALTER TABLE accounts ADD COLUMN IF NOT EXISTS profile_dir TEXT"#)
        .execute(&pool)
        .await?;

    sqlx::query(
        r#"ALTER TABLE accounts ADD COLUMN IF NOT EXISTS session_status TEXT DEFAULT 'unknown'"#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(r#"ALTER TABLE accounts ADD COLUMN IF NOT EXISTS session_error TEXT"#)
        .execute(&pool)
        .await?;

    sqlx::query(r#"ALTER TABLE accounts ADD COLUMN IF NOT EXISTS session_checked_at TIMESTAMPTZ"#)
        .execute(&pool)
        .await?;

    sqlx::query(r#"ALTER TABLE accounts ADD COLUMN IF NOT EXISTS cookies_synced_at TIMESTAMPTZ"#)
        .execute(&pool)
        .await?;

    sqlx::query(r#"CREATE INDEX IF NOT EXISTS idx_api_keys_user ON api_keys(user_id)"#)
        .execute(&pool)
        .await?;

    sqlx::query(r#"CREATE INDEX IF NOT EXISTS idx_api_keys_plan ON api_keys(plan_id)"#)
        .execute(&pool)
        .await?;

    // Modify usage_logs table
    sqlx::query(
        r#"ALTER TABLE usage_logs ADD COLUMN IF NOT EXISTS user_id INTEGER REFERENCES users(id) ON DELETE SET NULL"#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(r#"ALTER TABLE usage_logs ADD COLUMN IF NOT EXISTS request_kind TEXT"#)
        .execute(&pool)
        .await?;

    sqlx::query(r#"CREATE INDEX IF NOT EXISTS idx_usage_logs_user ON usage_logs(user_id)"#)
        .execute(&pool)
        .await?;

    sqlx::query(
        r#"CREATE INDEX IF NOT EXISTS idx_usage_logs_user_kind_model_created
           ON usage_logs(user_id, request_kind, model, created_at DESC)"#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"CREATE INDEX IF NOT EXISTS idx_usage_logs_api_key_kind_model_created
           ON usage_logs(api_key_id, request_kind, model, created_at DESC)"#,
    )
    .execute(&pool)
    .await?;

    // Transactions table
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS transactions (
            id SERIAL PRIMARY KEY,
            user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            type TEXT NOT NULL,
            method TEXT,
            amount NUMERIC(10,2) NOT NULL,
            currency TEXT DEFAULT 'USD',
            status TEXT DEFAULT 'pending',
            reference TEXT,
            proof_url TEXT,
            notes TEXT,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        )"#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(r#"CREATE INDEX IF NOT EXISTS idx_transactions_user ON transactions(user_id)"#)
        .execute(&pool)
        .await?;

    sqlx::query(r#"CREATE INDEX IF NOT EXISTS idx_transactions_status ON transactions(status)"#)
        .execute(&pool)
        .await?;

    // Migration 003: Providers table
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS providers (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            slug TEXT NOT NULL UNIQUE,
            active BOOLEAN DEFAULT true,
            created_at TIMESTAMPTZ DEFAULT NOW()
        )"#,
    )
    .execute(&pool)
    .await?;

    // Migration 003: Models table
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS models (
            id SERIAL PRIMARY KEY,
            provider_id INTEGER NOT NULL REFERENCES providers(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            slug TEXT NOT NULL UNIQUE,
            description TEXT,
            active BOOLEAN DEFAULT true,
            sort_order INTEGER DEFAULT 0,
            created_at TIMESTAMPTZ DEFAULT NOW()
        )"#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(r#"CREATE INDEX IF NOT EXISTS idx_models_provider ON models(provider_id)"#)
        .execute(&pool)
        .await?;

    sqlx::query(r#"CREATE INDEX IF NOT EXISTS idx_models_slug ON models(slug)"#)
        .execute(&pool)
        .await?;

    sqlx::query(r#"ALTER TABLE models ADD COLUMN IF NOT EXISTS description TEXT"#)
        .execute(&pool)
        .await?;

    // Migration 003: Plan-Model associations table
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS plan_models (
            plan_id INTEGER NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
            model_id INTEGER NOT NULL REFERENCES models(id) ON DELETE CASCADE,
            PRIMARY KEY (plan_id, model_id)
        )"#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(r#"CREATE INDEX IF NOT EXISTS idx_plan_models_plan ON plan_models(plan_id)"#)
        .execute(&pool)
        .await?;

    sqlx::query(r#"CREATE INDEX IF NOT EXISTS idx_plan_models_model ON plan_models(model_id)"#)
        .execute(&pool)
        .await?;

    // Migration 004: Conversations and media history
    sqlx::raw_sql(include_str!("../../migrations/004_conversations_media.sql"))
        .execute(&pool)
        .await?;

    // Ensure the real Grok imagine model exists and inherits access from grok-3 plans.
    sqlx::query(
        r#"
        INSERT INTO models (provider_id, name, slug, description, active, sort_order)
        SELECT p.id, 'Imagine X1', 'imagine-x-1', 'Fast image generation for everyday creative work.', true, 15
        FROM providers p
        WHERE p.slug = 'grok'
        ON CONFLICT (slug) DO UPDATE
        SET name = EXCLUDED.name, description = EXCLUDED.description, active = true, sort_order = EXCLUDED.sort_order
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        UPDATE models
        SET description = CASE slug
            WHEN 'grok-3' THEN 'Balanced default model for fast everyday chat.'
            WHEN 'grok-3-thinking' THEN 'Extra reasoning depth for harder prompts.'
            WHEN 'grok-4' THEN 'Higher quality answers for demanding tasks.'
            WHEN 'grok-4-auto' THEN 'Auto-tuned Grok 4 mode for mixed workloads.'
            WHEN 'grok-4-thinking' THEN 'Deep reasoning variant for complex problem solving.'
            WHEN 'imagine-x-1' THEN 'Fast image generation for everyday creative work.'
            ELSE description
        END
        WHERE description IS NULL
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO plan_models (plan_id, model_id)
        SELECT DISTINCT pm.plan_id, target.id
        FROM plan_models pm
        JOIN models base ON base.id = pm.model_id
        JOIN models target ON target.slug = 'imagine-x-1'
        WHERE base.slug = 'grok-3'
        ON CONFLICT (plan_id, model_id) DO NOTHING
        "#,
    )
    .execute(&pool)
    .await?;

    // Migration 004: Conversations table
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS conversations (
            id SERIAL PRIMARY KEY,
            user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            title TEXT,
            model_slug TEXT,
            active BOOLEAN DEFAULT true,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        )"#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"CREATE INDEX IF NOT EXISTS idx_conversations_user_created
           ON conversations(user_id, created_at DESC)"#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"CREATE INDEX IF NOT EXISTS idx_conversations_user_active
           ON conversations(user_id, active)"#,
    )
    .execute(&pool)
    .await?;

    // Migration 004: Messages table
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS messages (
            id SERIAL PRIMARY KEY,
            conversation_id INTEGER NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            model_slug TEXT,
            provider_slug TEXT,
            tokens_used INTEGER DEFAULT 0,
            created_at TIMESTAMPTZ DEFAULT NOW()
        )"#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"CREATE INDEX IF NOT EXISTS idx_messages_conversation_created
           ON messages(conversation_id, created_at ASC)"#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(r#"ALTER TABLE messages ADD COLUMN IF NOT EXISTS model_slug TEXT"#)
        .execute(&pool)
        .await?;

    sqlx::query(r#"ALTER TABLE messages ADD COLUMN IF NOT EXISTS provider_slug TEXT"#)
        .execute(&pool)
        .await?;

    // Migration 004: Image generation history
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS image_generations (
            id SERIAL PRIMARY KEY,
            user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            prompt TEXT NOT NULL,
            model_slug TEXT DEFAULT 'imagine-x-1',
            status TEXT DEFAULT 'pending',
            result_urls JSONB DEFAULT '[]'::jsonb,
            error_message TEXT,
            active BOOLEAN DEFAULT true,
            created_at TIMESTAMPTZ DEFAULT NOW()
        )"#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"CREATE INDEX IF NOT EXISTS idx_image_generations_user_created
           ON image_generations(user_id, created_at DESC)"#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"CREATE INDEX IF NOT EXISTS idx_image_generations_user_active
           ON image_generations(user_id, active)"#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"CREATE INDEX IF NOT EXISTS idx_image_generations_status
           ON image_generations(status)"#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS conversations (
            id SERIAL PRIMARY KEY,
            user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            title VARCHAR(255),
            model_slug VARCHAR(100),
            active BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )"#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"CREATE INDEX IF NOT EXISTS idx_conversations_user_created
           ON conversations(user_id, created_at DESC)"#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"CREATE INDEX IF NOT EXISTS idx_conversations_user_updated
           ON conversations(user_id, updated_at DESC)"#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS messages (
            id SERIAL PRIMARY KEY,
            conversation_id INTEGER NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
            role VARCHAR(20) NOT NULL,
            content TEXT NOT NULL,
            tokens_used INTEGER NOT NULL DEFAULT 0,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )"#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"CREATE INDEX IF NOT EXISTS idx_messages_conversation_created
           ON messages(conversation_id, created_at ASC)"#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS image_generations (
            id SERIAL PRIMARY KEY,
            user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            prompt TEXT NOT NULL,
            model_slug VARCHAR(100) NOT NULL DEFAULT 'imagine-x-1',
            status VARCHAR(20) NOT NULL DEFAULT 'pending',
            result_urls JSONB NOT NULL DEFAULT '[]'::jsonb,
            error_message TEXT,
            active BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )"#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"CREATE INDEX IF NOT EXISTS idx_image_generations_user_created
           ON image_generations(user_id, created_at DESC)"#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"CREATE INDEX IF NOT EXISTS idx_image_generations_status
           ON image_generations(status)"#,
    )
    .execute(&pool)
    .await?;

    run_media_studio_migrations(&pool).await?;

    tracing::info!("Database migrations completed successfully");
    Ok(())
}

async fn run_media_studio_migrations(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS conversations (
            id SERIAL PRIMARY KEY,
            user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            title VARCHAR(255),
            model_slug VARCHAR(100),
            active BOOLEAN DEFAULT true,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            updated_at TIMESTAMPTZ DEFAULT NOW()
        )"#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"CREATE INDEX IF NOT EXISTS idx_conversations_user_created
           ON conversations(user_id, created_at DESC)"#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"CREATE INDEX IF NOT EXISTS idx_conversations_user_active
           ON conversations(user_id, active)"#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS messages (
            id SERIAL PRIMARY KEY,
            conversation_id INTEGER NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
            role VARCHAR(20) NOT NULL,
            content TEXT NOT NULL,
            tokens_used INTEGER DEFAULT 0,
            created_at TIMESTAMPTZ DEFAULT NOW()
        )"#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"CREATE INDEX IF NOT EXISTS idx_messages_conversation_created
           ON messages(conversation_id, created_at ASC)"#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS image_generations (
            id SERIAL PRIMARY KEY,
            user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            prompt TEXT NOT NULL,
            model_slug VARCHAR(100) DEFAULT 'imagine-x-1',
            status VARCHAR(20) DEFAULT 'pending',
            result_urls JSONB DEFAULT '[]'::jsonb,
            error_message TEXT,
            created_at TIMESTAMPTZ DEFAULT NOW()
        )"#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"CREATE INDEX IF NOT EXISTS idx_image_generations_user_created
           ON image_generations(user_id, created_at DESC)"#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"CREATE INDEX IF NOT EXISTS idx_image_generations_status
           ON image_generations(status)"#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
