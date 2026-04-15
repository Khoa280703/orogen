CREATE TABLE IF NOT EXISTS conversations (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(255),
    model_slug VARCHAR(100),
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_conversations_user_created
    ON conversations(user_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_conversations_user_active
    ON conversations(user_id, active);

CREATE TABLE IF NOT EXISTS messages (
    id SERIAL PRIMARY KEY,
    conversation_id INTEGER NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL,
    content TEXT NOT NULL,
    model_slug TEXT,
    provider_slug TEXT,
    tokens_used INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_messages_conversation_created
    ON messages(conversation_id, created_at ASC);

ALTER TABLE messages ADD COLUMN IF NOT EXISTS model_slug TEXT;
ALTER TABLE messages ADD COLUMN IF NOT EXISTS provider_slug TEXT;

CREATE TABLE IF NOT EXISTS image_generations (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    prompt TEXT NOT NULL,
    model_slug VARCHAR(100) DEFAULT 'imagine-x-1',
    status VARCHAR(20) DEFAULT 'pending',
    result_urls JSONB DEFAULT '[]'::jsonb,
    error_message TEXT,
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_image_generations_user_created
    ON image_generations(user_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_image_generations_status
    ON image_generations(status);
