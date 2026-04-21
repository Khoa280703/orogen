# Phase 02 - Add Responses-Compatible API

## Context Links
- Plan overview: `plans/260420-1152-codex-account-pool-api-gateway/plan.md`
- Current OpenAI chat endpoint: `src/api/chat_completions.rs`
- API router: `src/api/mod.rs`
- 9router architecture: `9router/docs/ARCHITECTURE.md`
- 9router Codex CLI config path: `9router/src/app/api/cli-tools/codex-settings/route.js`

## Overview
- Priority: Critical
- Status: Completed
- Brief: Expose the API surface that Codex CLI and similar tools actually need, with `responses` as first-class wire API.

## Key Insights
- Codex CLI integration should target a compatibility endpoint owned by `duanai`, not internal accounts.
- `chat/completions` should stay for broader client compatibility, but `responses` should be the primary Codex-facing contract.
- `duanai` already has translation-friendly provider abstractions and SSE handling primitives that can be extended.
- Public API handlers should not know or care which upstream provider actually serves the request.

## Requirements
- Support `POST /v1/responses`.
- Keep `POST /v1/chat/completions`.
- Keep `GET /v1/models`.
- Public contract must remain stable even if internal upstream serving changes.
- Public contract must support provider-agnostic routing under the same API key/plan layer.

## Architecture
- Add `responses` request/response types parallel to current chat completions.
- Normalize both `chat.completions` and `responses` into one shared internal request representation.
- Return SSE or JSON depending on requested mode and endpoint contract.
- Preserve plan enforcement using resolved public model slug.
- Define the canonical `public model -> route resolver` interface in this phase so endpoint handlers never read raw provider model rows directly.
- Keep compatibility translation modular so future Anthropic-compatible or Gemini-style entrypoints can sit beside the same core.
- Do not let endpoint handlers speak directly to provider adapters. They should call:
- normalize request
- resolve public model
- resolve provider route
- execute through orchestration core
- format response for requested wire API

## Related Code Files
- Modify: `src/api/mod.rs`
- Modify: `src/api/chat_completions.rs`
- Create or modify: `src/api/responses.rs`
- Create or modify: `src/api/request_orchestrator.rs`
- Modify: `src/providers/types.rs`
- Modify: `src/api/models.rs`

## Implementation Steps
1. Introduce `/v1/responses` route and typed handler.
2. Define a shared normalized internal request representation for chat-like workloads.
3. Create a shared orchestration core so `/chat/completions`, `/responses`, and future endpoints do not duplicate provider-routing logic.
4. Introduce a stable route-resolution boundary that can temporarily read current schema, but becomes the only path handlers use.
5. Re-use plan enforcement and usage recording from current chat path.
6. Normalize outgoing SSE framing and terminal JSON response for `responses`.
7. Keep unsupported fields explicit rather than silently swallowing everything.

## Todo List
- [x] Add `responses` route
- [x] Add internal normalization layer
- [x] Add shared orchestration core
- [x] Add public-model route resolver boundary
- [x] Re-use plan enforcement and usage logging
- [x] Document public differences between `responses` and `chat/completions`

## Success Criteria
- Codex CLI can point to `duanai` using a `responses` wire API.
- Existing OpenAI-compatible clients keep working on `chat/completions`.
- Usage and plan limits remain consistent across both endpoints.

## Risk Assessment
- Mixed endpoint semantics can create duplicated translation logic if not centralized.
- SSE event shape mismatches can break CLI clients in subtle ways.

## Security Considerations
- Validate model access before any upstream request starts.
- Avoid leaking internal provider/account metadata in public API errors.

## Next Steps
- After public compatibility exists, strengthen internal account routing and fairness controls.

Resolution:
- Resolved for v1: `responses` stays text-first. Top-level `tools` declarations are accepted and ignored for client compatibility, while actual tool-calling/computer-call payloads and image input are still rejected.
