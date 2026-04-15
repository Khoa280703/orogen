# Phase 2: Backend Provider Abstraction

## Context
- [Phase 1 — DB](./phase-01-db-conversations-media.md)
- Current: `src/grok/` module tightly coupled — client.rs, imagine_ws.rs, types.rs all Grok-specific
- Goal: abstract behind traits so future providers (Flux, SD, DALL-E) slot in without touching consumer API

## Overview
- **Priority**: P1 (blocker for Phase 3)
- **Status**: Complete
- **Effort**: 5h

Create `ChatProvider` + `ImageProvider` traits (VideoProvider deferred — MVP only). Grok module implements both. Consumer API codes against traits, not Grok directly.

## Key Insights
- Current `GrokClient` is stateless (`pub struct GrokClient;`) — easy to wrap
- Account pool rotation logic lives in API handlers, not client — keep it there
- Image gen uses WebSocket (imagine_ws), chat uses REST — different protocols but same trait interface
- Provider trait should NOT own account selection — that's orchestration layer's job

## Architecture

```
Consumer API (Phase 3)
    ↓
Provider Registry (maps provider slug → impl)
    ↓
┌─────────────────┐  ┌──────────────────┐
│ GrokChatProvider │  │ GrokImageProvider │
│ (REST + SSE)     │  │ (WebSocket)       │
└─────────────────┘  └──────────────────┘
    ↓                     ↓
Account Pool + Proxy (existing, unchanged)
```

### Traits

```rust
// src/providers/chat_provider.rs
#[async_trait]
pub trait ChatProvider: Send + Sync {
    /// Stream chat response, sending events via channel
    async fn chat_stream(
        &self,
        cookies: &GrokCookies,
        proxy_url: Option<&String>,
        model: &str,
        messages: &[ChatMessage],
        system_prompt: &str,
    ) -> Result<mpsc::Receiver<ChatStreamEvent>, ProviderError>;
}

// src/providers/image_provider.rs
#[async_trait]
pub trait ImageProvider: Send + Sync {
    async fn generate_images(
        &self,
        cookies: &GrokCookies,
        proxy_url: Option<&String>,
        prompt: &str,
        model: &str,
    ) -> Result<Vec<GeneratedAsset>, ProviderError>;
}
```

### Shared Types

```rust
// src/providers/types.rs
pub struct ChatMessage {
    pub role: String,     // user | assistant | system
    pub content: String,
}

pub enum ChatStreamEvent {
    Token(String),
    Thinking(String),
    Done,
    Error(String),
}

pub struct GeneratedAsset {
    pub id: String,
    pub url: String,
}

pub enum ProviderError {
    RateLimited,
    Unauthorized,
    NetworkError(String),
    Blocked(String),
}
```

## Related Code Files

| File | Action | Purpose |
|------|--------|---------|
| `src/providers/mod.rs` | CREATE | Module + provider registry |
| `src/providers/types.rs` | CREATE | Shared types (ChatMessage, ChatStreamEvent, etc.) |
| `src/providers/chat_provider.rs` | CREATE | ChatProvider trait |
| `src/providers/image_provider.rs` | CREATE | ImageProvider trait |
| `src/providers/grok_chat.rs` | CREATE | Grok ChatProvider impl (wraps existing client.rs) |
| `src/providers/grok_image.rs` | CREATE | Grok ImageProvider impl (wraps existing imagine_ws.rs) |
| `src/main.rs` | MODIFY | Add `mod providers;`, register providers in AppState |
| `src/grok/*` | KEEP | Unchanged — providers wrap these, don't replace |

## Implementation Steps

1. Create `src/providers/types.rs` (~40 lines):
   - ChatMessage, ChatStreamEvent, GeneratedAsset, ProviderError
   - Conversion from ProviderError → AppError

2. Create `src/providers/chat_provider.rs` (~15 lines):
   - `ChatProvider` trait definition

3. Create `src/providers/image_provider.rs` (~15 lines):
   - `ImageProvider` trait definition

4. Create `src/providers/grok_chat.rs` (~80 lines):
   - `GrokChatProvider` struct (wraps existing `GrokClient`)
   - Implements `ChatProvider` trait
   - Converts ChatMessage → GrokRequest
   - Wraps `grok::client::stream_chat()` → ChatStreamEvent channel
   - Does NOT handle account rotation (caller's job)

5. Create `src/providers/grok_image.rs` (~50 lines):
   - `GrokImageProvider` struct
   - Implements `ImageProvider` trait
   - Wraps `grok::imagine_ws::generate_images()`
   - Converts result → Vec<GeneratedAsset>

6. Create `src/providers/mod.rs` (~30 lines):
   - Re-export traits + types
   - `ProviderRegistry` struct: `HashMap<String, (Box<dyn ChatProvider>, Box<dyn ImageProvider>)>`
   - `get_chat_provider(slug)`, `get_image_provider(slug)`
   - Register "grok" provider on init

7. Update `src/main.rs`:
   - Add `mod providers;`
   - Add `providers: ProviderRegistry` to AppState
   - Initialize GrokChatProvider + GrokImageProvider on startup

## Todo List
- [x] Create providers/types.rs
- [x] Create providers/chat_provider.rs trait
- [x] Create providers/image_provider.rs trait
- [x] Create providers/grok_chat.rs implementation
- [x] Create providers/grok_image.rs implementation
- [x] Create providers/mod.rs with registry
- [x] Update main.rs AppState
- [x] `cargo build` to verify

## Success Criteria
- All provider files compile
- GrokChatProvider wraps existing chat flow correctly
- GrokImageProvider wraps existing imagine_ws correctly
- ProviderRegistry resolves "grok" to both providers
- Existing `/v1/*` endpoints still work (no breakage)

## Risk Assessment
- **Medium**: wrapping existing async streaming code in trait requires careful lifetime management
- `async_trait` crate handles `async fn` in traits
- Account rotation stays in API handler layer — no risk of breaking pool logic

## Security Considerations
- Provider trait receives cookies/proxy — same security model as current
- No new attack surface

## Next Steps
- Phase 3 uses these traits for consumer API endpoints
