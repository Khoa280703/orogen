# Phase 1: DB Migration — Conversations & Media Tables

## Context
- [Brainstorm](../reports/brainstorm-260409-0947-media-studio-pivot.md)
- Current migrations: `001_initial_schema.sql`, `002_users_plans.sql`, `003_providers_models.sql`

## Overview
- **Priority**: P1 (blocker for all other phases)
- **Status**: Complete
- **Effort**: 3h

New tables for conversation persistence and media generation history.

## Key Insights
- Chat needs conversation threads (multi-turn)
- Image gen needs generation history with prompt + results
- Both link to user_id for billing/usage tracking
- Media URLs come from Grok (no self-hosted storage in MVP)

## Requirements

### Functional
- Store conversations with messages (user + assistant)
- Store image generation requests with results (URLs, metadata)
- Link all to user_id + plan enforcement
- Support pagination for history listing

### Non-functional
- Indexes on user_id + created_at for fast lookups
- Soft-delete support (active flag)

## Architecture

```sql
-- Conversation threads
conversations (
  id SERIAL PRIMARY KEY,
  user_id INT NOT NULL REFERENCES users(id),
  title VARCHAR(255),          -- auto-generated or user-set
  model_slug VARCHAR(100),     -- which model used
  active BOOLEAN DEFAULT true,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
)

-- Messages within conversations
messages (
  id SERIAL PRIMARY KEY,
  conversation_id INT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
  role VARCHAR(20) NOT NULL,   -- 'user' | 'assistant' | 'system'
  content TEXT NOT NULL,
  -- NOTE: thinking content NOT stored (stream-only, not persisted)
  tokens_used INT DEFAULT 0,
  created_at TIMESTAMPTZ DEFAULT NOW()
)

-- Image generation history
image_generations (
  id SERIAL PRIMARY KEY,
  user_id INT NOT NULL REFERENCES users(id),
  prompt TEXT NOT NULL,
  model_slug VARCHAR(100) DEFAULT 'imagine-x-1',
  status VARCHAR(20) DEFAULT 'pending',  -- pending | completed | failed
  result_urls JSONB DEFAULT '[]',        -- array of {id, url}
  error_message TEXT,
  created_at TIMESTAMPTZ DEFAULT NOW()
)
```

## Related Code Files

| File | Action | Purpose |
|------|--------|---------|
| `migrations/004_conversations_media.sql` | CREATE | New tables + indexes |
| `src/db/migrate.rs` | MODIFY | Add migration 004 |
| `src/db/conversations.rs` | CREATE | Conversation CRUD |
| `src/db/messages.rs` | CREATE | Message CRUD |
| `src/db/image_generations.rs` | CREATE | Image gen history CRUD |
| `src/db/mod.rs` | MODIFY | Register new modules |

## Implementation Steps

1. Create `migrations/004_conversations_media.sql`:
   - conversations table + indexes (user_id, created_at)
   - messages table + indexes (conversation_id, created_at)
   - image_generations table + indexes (user_id, created_at, status)

2. Update `src/db/migrate.rs` — add migration 004 to migration list

3. Create `src/db/conversations.rs` (~80 lines):
   - `create_conversation(db, user_id, title, model_slug) -> Conversation`
   - `list_conversations(db, user_id, limit, offset) -> Vec<Conversation>`
   - `get_conversation(db, id, user_id) -> Option<Conversation>`
   - `update_title(db, id, title)`
   - `delete_conversation(db, id, user_id)` — soft delete

4. Create `src/db/messages.rs` (~60 lines):
   - `create_message(db, conversation_id, role, content, thinking) -> Message`
   - `list_messages(db, conversation_id) -> Vec<Message>`
   - `count_messages(db, conversation_id) -> i64`

5. Create `src/db/image_generations.rs` (~70 lines):
   - `create_generation(db, user_id, prompt, model_slug) -> ImageGeneration`
   - `update_generation_result(db, id, urls) -> update status=completed`
   - `update_generation_error(db, id, error) -> update status=failed`
   - `list_generations(db, user_id, limit, offset) -> Vec<ImageGeneration>`
   - `get_generation(db, id, user_id) -> Option<ImageGeneration>`

6. Update `src/db/mod.rs` — add `pub mod conversations; pub mod messages; pub mod image_generations;`

## Todo List
- [x] Create migration 004
- [x] Update migrate.rs
- [x] Create conversations.rs
- [x] Create messages.rs
- [x] Create image_generations.rs
- [x] Update db/mod.rs
- [x] Run `cargo build` to verify

## Success Criteria
- Migration runs on server start without errors
- All CRUD functions compile
- Tables created with proper indexes and foreign keys

## Risk Assessment
- **Low**: straightforward SQL migration, follows existing patterns (001-003)
- Migration should be idempotent (IF NOT EXISTS)

## Next Steps
- Phase 2: Provider abstraction (uses these tables for persistence)
- Phase 3: Consumer API (exposes these tables via endpoints)
