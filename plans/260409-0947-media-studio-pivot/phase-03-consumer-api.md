# Phase 3: Backend Consumer API — Chat + Image Endpoints

## Context
- [Phase 1 — DB](./phase-01-db-conversations-media.md)
- [Phase 2 — Provider Abstraction](./phase-02-provider-abstraction.md)
- Current `/v1/*` routes: OpenAI-compatible format, developer-facing
- New: `/api/chat/*` and `/api/images/*` routes for consumer web UI

## Overview
- **Priority**: P1 (blocker for frontend phases 4, 5)
- **Status**: Complete
- **Effort**: 6h

Consumer-facing REST API for chat and image generation. JWT-authenticated (reuse existing user auth). Conversations persisted to DB. Image generations tracked with history.

## Key Insights
- Consumer API is separate from `/v1/*` (which stays for developer API keys)
- Consumer uses JWT auth (cookies), not API key auth
- Chat needs SSE streaming endpoint (same pattern as chat_completions but simpler)
- Account rotation + plan enforcement reused from existing code
- Keep it simple: no Anthropic format, no tool calling, just text + images

## Architecture

### New Routes

```
POST   /api/chat/conversations              → create conversation
GET    /api/chat/conversations              → list conversations
GET    /api/chat/conversations/:id          → get conversation with messages
DELETE /api/chat/conversations/:id          → delete conversation
POST   /api/chat/conversations/:id/messages → send message (SSE stream response)

POST   /api/images/generate                 → generate images
GET    /api/images/history                  → list generation history
GET    /api/images/history/:id             → get single generation
```

### Request/Response

```rust
// Chat message send
POST /api/chat/conversations/:id/messages
{
  "content": "Hello",
  "model": "grok-3"  // optional, defaults to conversation model
}
// Response: SSE stream
// event: token\ndata: {"content": "Hi"}\n\n
// event: thinking\ndata: {"content": "..."}\n\n
// event: done\ndata: {}\n\n

// Image generate
POST /api/images/generate
{
  "prompt": "a cat in space",
  "model": "imagine-x-1"  // optional
}
// Response: JSON
{
  "id": 123,
  "prompt": "a cat in space",
  "status": "completed",
  "images": [{"id": "...", "url": "..."}]
}
```

## Related Code Files

| File | Action | Purpose |
|------|--------|---------|
| `src/api/consumer_chat.rs` | CREATE | Chat conversation endpoints |
| `src/api/consumer_images.rs` | CREATE | Image generation endpoints |
| `src/api/mod.rs` | MODIFY | Register new `/api/chat/*` and `/api/images/*` routes |
| `src/api/plan_enforcement.rs` | MODIFY | Add helper for JWT-user enforcement (not just API key) |

## Implementation Steps

1. Create `src/api/consumer_chat.rs` (~180 lines):

   a. **create_conversation** — POST /api/chat/conversations
      - Extract JWT user_id
      - Enforce plan access for requested model
      - Insert into conversations table
      - Return conversation object

   b. **list_conversations** — GET /api/chat/conversations
      - Extract JWT user_id
      - Query conversations with pagination (limit/offset query params)
      - Return list with message_count

   c. **get_conversation** — GET /api/chat/conversations/:id
      - Verify ownership (user_id match)
      - Return conversation + all messages

   d. **delete_conversation** — DELETE /api/chat/conversations/:id
      - Soft delete (set active=false)

   e. **send_message** — POST /api/chat/conversations/:id/messages
      - Verify ownership
      - Enforce plan access + quota
      - Save user message to DB
      - Get account from pool
      - Build ChatMessage history from DB messages
      - Call provider.chat_stream() via ProviderRegistry
      - Stream SSE response (token/thinking/done events)
      - On stream complete: save assistant message to DB
      - Record usage log
      - Handle retry on rate limit (rotate account)

2. Create `src/api/consumer_images.rs` (~120 lines):

   a. **generate_images** — POST /api/images/generate
      - Extract JWT user_id
      - Enforce plan access + quota
      - Create image_generations record (status=pending)
      - Get account from pool
      - Call provider.generate_images()
      - Update record with result URLs (status=completed) or error (status=failed)
      - Record usage log
      - Return generation object

   b. **list_history** — GET /api/images/history
      - Extract JWT user_id
      - Query with pagination
      - Return list

   c. **get_generation** — GET /api/images/history/:id
      - Verify ownership
      - Return generation details

3. Update `src/api/mod.rs`:
   - Add `pub mod consumer_chat; pub mod consumer_images;`
   - Add route group:
     ```rust
     let consumer_routes = Router::new()
         .route("/chat/conversations", post(consumer_chat::create_conversation))
         .route("/chat/conversations", get(consumer_chat::list_conversations))
         .route("/chat/conversations/:id", get(consumer_chat::get_conversation))
         .route("/chat/conversations/:id", delete(consumer_chat::delete_conversation))
         .route("/chat/conversations/:id/messages", post(consumer_chat::send_message))
         .route("/images/generate", post(consumer_images::generate_images))
         .route("/images/history", get(consumer_images::list_history))
         .route("/images/history/:id", get(consumer_images::get_generation));
     ```
   - Nest under `/api`: `.nest("/api", consumer_routes)`
   - Apply JWT middleware to consumer routes

4. Update `src/api/plan_enforcement.rs` (~20 lines added):
   - Add `enforce_user_plan_access(db, user_id, model_slug)` — for JWT users (no api_key_id)
   - Reuses existing logic but simplified path

## Todo List
- [x] Create consumer_chat.rs
- [x] Create consumer_images.rs
- [x] Update mod.rs with routes
- [x] Update plan_enforcement.rs
- [x] `cargo build` to verify
- [x] Test with JWT-authenticated smoke user: create conversation, send message, generate image

## Success Criteria
- All endpoints compile and respond correctly
- Chat SSE streaming works end-to-end
- Image generation creates record + returns URLs
- Plan enforcement blocks unauthorized model access
- Quota counting works for consumer endpoints
- Conversation history persists across requests

## Risk Assessment
- **Medium**: SSE streaming with DB persistence — need to handle partial stream failures (save what we have)
- **Low**: Image gen is basically existing flow with DB tracking added
- Account rotation logic copied from chat_completions.rs — well-tested pattern
- **Residual risk**: current local Grok account credentials are invalid upstream, so live generation/message send still returns external-service errors after the consumer API hands off correctly

## Security Considerations
- All consumer endpoints require valid JWT
- Conversation ownership enforced (user_id match)
- Rate limiting inherited from existing middleware
- No direct cookie/account exposure to consumers

## Next Steps
- Phase 4: Chat UI consumes `/api/chat/*`
- Phase 5: Image Studio consumes `/api/images/*`

## Current Progress
- JWT-authenticated smoke test completed against the latest backend build
- `/api/chat/conversations` list/create now work end-to-end for a real user session
- `/api/images/history` list works and persists failed generations correctly
- `send_message` and `generate_images` both reach provider execution, but fail at upstream Grok authentication, confirming the remaining blocker is external credentials rather than missing consumer API implementation
