# Phase 01 - Realign Codex Provider Runtime

## Context Links
- Plan overview: `plans/260420-1152-codex-account-pool-api-gateway/plan.md`
- Existing Codex auth flow: `src/services/codex_oauth.rs`
- Existing provider wiring: `src/providers/mod.rs`, `src/providers/codex_chat.rs`
- Existing temporary bridge: `src/services/codex_client.rs`
- 9router references: `9router/open-sse/config/providers.js`, `9router/open-sse/executors/codex.js`

## Overview
- Priority: Critical
- Status: Completed
- Brief: Replace native CLI-per-request serving with a real Codex upstream executor model, while defining the provider contract that future Gemini/Grok/Claude adapters must follow.

## Implementation Status
- Direct Codex upstream HTTP execution is now the active serving path; request-time native CLI spawning is no longer the runtime model.
- Provider-specific account preparation now hangs off the provider hook instead of being hard-coded inside `AccountPool`, so Codex token refresh remains provider-owned and the next OAuth/API-key provider does not need another pool-level special case.
- Current proxy assignment and pool-routing semantics are now pinned by the route-level coverage that landed later in the plan, so Phase 1 no longer depends on hidden runtime assumptions to stay correct.

## Key Insights
- Native CLI spawning works as a tactical patch, but it is the wrong serving primitive for large account pools.
- `duanai` already has provider abstraction and account pool logic; the missing piece is a Codex executor that behaves like a first-class upstream provider.
- 9router pattern shows Codex can be handled as a direct HTTP upstream with OAuth refresh and compatibility translation layered above it.
- This phase must produce a reusable provider-executor contract, not a Codex-only special path.

## Requirements
- Codex serving path must not depend on local `codex` CLI binary at request time.
- Codex account refresh lifecycle must remain inside `duanai`.
- Runtime must support per-account proxying and future cooldown/failover.
- The executor interface must be reusable for providers with different auth modes and endpoint formats.

## Architecture
- Keep Codex OAuth onboarding and token persistence.
- Refactor `src/services/codex_client.rs` into a direct upstream executor/client.
- Keep `src/providers/codex_chat.rs` thin and provider-oriented.
- Add an explicit boundary between:
- auth lifecycle
- request translation/execution
- account pool routing
- Reconfirm provider contract layers:
- provider auth/account preparation
- model capability description
- request transform
- upstream execution
- stream/result normalization
- Freeze a provider adapter contract before broader rollout. Each provider adapter must declare:
- auth/account preparation hook
- request format(s) it accepts
- compatibility surfaces it can serve
- streaming support
- proxy support
- refresh strategy
- error classification rules

## Related Code Files
- Modify: `src/services/codex_client.rs`
- Modify: `src/providers/codex_chat.rs`
- Modify: `src/providers/mod.rs`
- Modify: `src/main.rs`
- Modify: `src/account/pool.rs`
- Create or modify: `src/providers/types.rs`
- Review only: `src/services/codex_oauth.rs`

## Implementation Steps
1. Remove native CLI request execution from Codex serving path.
2. Implement direct Codex upstream client with request builder, headers, SSE parsing, and auth error classification.
3. Extract or formalize the provider executor contract so later Gemini/Grok/Claude providers fit the same runtime shape.
4. Keep Codex OAuth refresh in `src/services/codex_oauth.rs` and re-use from account preparation.
5. Ensure provider errors classify into retryable vs non-retryable categories for pool logic.
6. Keep UI/admin flows unchanged in this phase unless runtime assumptions break them.

## Todo List
- [x] Replace CLI bridge in Codex runtime
- [x] Re-classify Codex error handling for pool routing
- [x] Verify account refresh still updates DB/session state
- [x] Keep build green after provider refactor

## Success Criteria
- Codex provider can serve requests without spawning native CLI.
- Existing account records remain usable after migration.
- Codex runtime fits the same provider lifecycle as Grok.
- A second provider can be onboarded later without inventing another serving abstraction.
- Provider adapter contract is explicit enough that onboarding docs can be written from it.

## Risk Assessment
- Upstream endpoint/header contract may differ from previous assumptions.
- Refresh token logic may appear valid while upstream serving still rejects a subset of scopes.
- SSE event drift can still surface as misleading timeout/transient errors if unknown upstream events are ignored too aggressively.
- Pool health signals still depend on route-level verification and future upstream contract drift monitoring, not only on compile-time types.

## Security Considerations
- Do not expose refresh/access tokens through debug responses.
- Ensure any new headers or session identifiers stay server-side only.

## Next Steps
- After runtime is stable, implement explicit `/v1/responses` public compatibility layer.

Resolution:
- None for Phase 1 acceptance.
