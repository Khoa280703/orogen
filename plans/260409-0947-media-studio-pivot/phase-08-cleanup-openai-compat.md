# Phase 8: Cleanup — Remove OpenAI Compatibility Layer

## Context
- Current: `/v1/chat/completions`, `/v1/messages` (Anthropic), tool calling, function call parsing
- These are the largest files: chat_completions.rs (1485L), anthropic_messages.rs (1067L)
- After pivot: consumers use `/api/chat/*`, developers can still use simplified `/v1/*`

## Overview
- **Priority**: P3
- **Status**: Complete
- **Effort**: 2h

Simplify `/v1/*` layer. Remove Anthropic format. Remove tool calling. Keep basic chat completions for developer API key users.

## Key Decision
**Do NOT delete `/v1/*` entirely** — developer users still need API access. But simplify dramatically:
- Remove tool calling / function call parsing
- Remove Anthropic messages endpoint
- Remove model alias resolution
- Simplify to: messages in → streaming text out

## Implementation Steps

1. **Simplify chat_completions.rs** (1485L → ~300L):
   - Remove tool call parsing and rebuilding
   - Remove function call format conversion
   - Remove complex retry logic (use provider abstraction instead)
   - Keep: message format parsing, SSE streaming, usage tracking
   - Use ChatProvider trait instead of direct GrokClient calls

2. **Remove anthropic_messages.rs** (1067L → DELETE):
   - Remove Anthropic API compatibility
   - Remove `/v1/messages` and `/v1/messages/count_tokens` routes

3. **Remove model_mapping.rs** (18L → DELETE):
   - Alias resolution already deprecated
   - Direct model slug from DB only

4. **Update image_generations.rs** — use ImageProvider trait

5. **Update video_generations.rs** — use provider abstraction (or keep as-is for now)

6. **Update mod.rs** — remove deleted routes

## Related Code Files

| File | Action | Purpose |
|------|--------|---------|
| `src/api/chat_completions.rs` | SIMPLIFY | Remove tool calling, use provider trait |
| `src/api/anthropic_messages.rs` | DELETE | No longer needed |
| `src/api/model_mapping.rs` | DELETE | Aliases removed |
| `src/api/image_generations.rs` | MODIFY | Use ImageProvider trait |
| `src/api/mod.rs` | MODIFY | Remove deleted routes |

## Todo List
- [x] Simplify chat_completions.rs
- [x] Delete anthropic_messages.rs
- [x] Delete model_mapping.rs
- [x] Update image_generations.rs
- [x] Update mod.rs
- [x] `cargo build` to verify
- [ ] Live test `/v1/chat/completions` basic chat against upstream Grok account

## Success Criteria
- `/v1/chat/completions` works for simple text chat (no tools)
- Anthropic endpoint removed
- Model aliases removed
- Code reduced by ~2000 lines
- All existing consumer features unaffected

## Current Progress
- `src/api/chat_completions.rs` is now 487 lines and uses `ChatProvider` via `start_chat_stream_with_retry` instead of the old direct OpenAI-compat/tool-call stack
- `/v1/chat/completions` now explicitly rejects `tools` plus `tool` and `function` role messages, leaving simple text/system message flows only
- `src/api/image_generations.rs` now resolves the provider through `get_image_provider("grok")` and shares the consumer retry/provider path
- `src/api/anthropic_messages.rs` is removed and `/v1/messages` plus `/v1/messages/count_tokens` are no longer registered in `src/api/mod.rs`
- `src/api/model_mapping.rs` is removed; model alias cleanup already landed and `/v1` now uses direct DB model slugs
- `src/api/video_generations.rs` remains on the direct Grok flow. This phase left it as-is per plan instead of forcing a provider abstraction rewrite

## Verification
- 2026-04-09: `cargo build` passed at repo root after cleanup changes
- 2026-04-09: `src/api/mod.rs` route inspection confirms `/v1/messages` routes are gone; remaining `/v1` endpoints are `/models`, `/chat/completions`, `/images/generations`, and `/videos/generations`
- 2026-04-09: repository scan confirms `src/api/anthropic_messages.rs` and `src/api/model_mapping.rs` no longer exist
- 2026-04-09: static handler inspection confirms basic text extraction remains covered, including the existing unit test for OpenAI content arrays in `src/api/chat_completions.rs`

## Blocker Status
- Phase 8 implementation: none
- Operational blocker: live end-to-end `/v1/chat/completions` smoke still depends on a working upstream Grok account; current local accounts return `Unauthorized` or `invalid-credentials`

## Risk Assessment
- **Medium**: removing tool calling breaks developer users who depend on it
- Mitigation: communicate breaking change, tools never worked well via scraping anyway
- Residual risk: `src/api/video_generations.rs` still uses the direct Grok client path, so provider abstraction cleanup is partial by design
