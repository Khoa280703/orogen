# Phase 05 - Observability, Cutover, And Cleanup

## Context Links
- Plan overview: `plans/260420-1152-codex-account-pool-api-gateway/plan.md`
- Current admin accounts UI: `web/src/app/admin/accounts/page.tsx`
- Usage logging: `src/db/usage_logs.rs`, `src/api/chat_completions.rs`
- Current temporary Codex runtime patch: `src/services/codex_client.rs`

## Overview
- Priority: High
- Status: Completed
- Brief: Cut over safely from tactical Codex experiments to a product-grade API gateway path.

## Key Insights
- The dangerous failure mode is silent drift: UI says one thing, pool does another, customer plans sell a third thing.
- Temporary compatibility hacks should be removed only after new path is observable and verified.
- Multi-provider gateways fail asymmetrically; observability must slice by provider, account cohort, customer plan, and endpoint shape, while model-field semantics are normalized deliberately instead of assumed.

## Requirements
- Clear admin visibility into:
- provider account health
- upstream error classes
- customer usage by key/plan/model
- public model catalog
- Safe rollback path during cutover.

## Architecture
- Add dashboard/admin slices for provider-specific routing health, starting with Codex.
- Tag usage logs with provider slug first; treat stable public-model observability as incomplete until every serving surface logs normalized public-model semantics.
- Keep migration flags or staged rollout toggles where necessary.
- Add verification gates per layer:
- provider adapter contract tests
- orchestration-core tests
- compatibility endpoint tests
- plan/catalog enforcement tests
- account routing failover tests

## Related Code Files
- Modify: `src/api/admin_accounts.rs`
- Modify: `src/api/admin_stats.rs`
- Modify: `web/src/app/admin/accounts/page.tsx`
- Modify: `web/src/app/admin/usage/*`
- Cleanup candidate: `src/services/codex_client.rs` temporary paths if superseded

## Implementation Steps
1. Add admin-readable health and routing counters for Codex pool.
2. Add endpoint verification matrix for:
- `/v1/models`
- `/v1/chat/completions`
- `/v1/responses`
- plan enforcement
- account fallback
3. Add tests/checklists that a new provider must pass before rollout.
4. Remove temporary runtime shortcuts that conflict with target serving model.
5. Update docs and operator runbook for customer support.

## Todo List
- [x] Add provider/model visibility to usage logs and admin usage filtering
- [x] Verify the remaining cutover matrix for the current serving contract, including route-level stream ordering and fail-closed recovery boundaries
- [x] Add provider onboarding verification gates
- [x] Remove obsolete bridge logic
- [x] Update support/deployment docs

## Implementation Status
- Landed a first verification slice for the documented Codex-facing path: backend helper/contract tests now cover public catalog response shaping in `/v1/models`, text-first request acceptance in `/v1/responses`, top-level tool-declaration compatibility on `/v1/responses`, rejection of unsupported tool/function message roles and image inputs on `/v1/responses`, and preservation of the public model slug in non-streamed response payloads.
- This does not close the full cutover matrix yet because it is still helper/unit contract coverage, not endpoint-level or end-to-end account-routing validation.
- Landed a second verification slice with DB-backed router tests for `/v1/models` and `/v1/responses`: the tests seed providers, public model routes, plan-to-public-model mapping, and API keys in a temporary PostgreSQL database, then verify actual HTTP routing reaches the expected auth, plan, payload-validation, and `NoAccounts` boundaries without calling any upstream provider.
- Landed a third verification slice with matching DB-backed router tests for `/v1/chat/completions`: the route now has explicit endpoint-level verification for auth, plan denial, unsupported tools/image/legacy tool-call payload rejection, and `NoAccounts` boundary behavior through the real HTTP stack without upstream traffic.
- Landed a fourth verification slice for `stream=true` on `/v1/chat/completions`: the route now has explicit SSE-path coverage showing that stream requests still fail closed behind auth/plan checks and surface the internal `NoAccounts` boundary without real upstream traffic.
- Landed a fifth verification slice for `stream=true` on `/v1/responses`: the Codex-facing Responses route now has explicit SSE-path coverage showing that stream requests still fail closed behind auth/plan checks and surface the internal `response.failed` boundary plus `[DONE]` without real upstream traffic.
- This still does not close the full cutover matrix because retry/failover behavior and broader operator checklists are not covered by the current router slices.
- Landed provider onboarding verification gates in admin health: active providers now expose adapter/capability presence, expected auth mode, selectable-account coverage, active public route count, active-plan assignment count, and rollout warnings so partial provider onboarding is visible before customer traffic depends on it.
- Updated support/deployment docs for the current customer-facing contract and minimum operator runbook: `/docs/api` now documents the text-first `/v1/responses` surface, current streaming failure semantics, production error payload shape, auth bootstrap behavior, and the 400/403/503 boundaries operators should expect, while `/docs/guides/provider-onboarding` now includes the required backend verification commands, admin-health acceptance criteria, rollback/support flow, and a cutover rule that router-level checks do not substitute for adapter/auth or retry/failover validation.
- Landed a sixth verification slice with a local Codex upstream stub and seeded multi-account pool: DB-backed ignored router tests now prove pre-stream failover works through the real router-to-local-stub path for both `/v1/chat/completions` and `stream=true` on `/v1/responses`, including rate-limit classification on the first account, successful fallback to the next account, and persisted routing-health updates on both accounts without touching the real upstream. This slice still verifies routing-state and fallback behavior, not the full upstream request-shape contract, and it still requires `DATABASE_URL` plus `CREATE/DROP DATABASE` access outside the default `cargo test` path.
- Landed a seventh verification slice on auth-expiry fallback: the same DB-backed ignored router path now also proves pre-stream `Unauthorized` classification expires the failed account, persists `auth_invalid` plus `session_status=expired`, and falls through to the next healthy account for both `/v1/chat/completions` and `stream=true` on `/v1/responses` before any SSE payload is emitted.
- Landed an eighth verification slice on proxy failover: DB-backed ignored router tests now prove proxy-classified pre-stream failures can deactivate the failed proxy persistently, reassign the affected account cohort onto a healthy replacement proxy, and recover the same request for both `/v1/chat/completions` and `stream=true` on `/v1/responses` before the first customer-visible SSE event. The slice now covers both `CfBlocked` and transport-level `ProxyFailed(connect refused)` on the chat route, and it also verifies that peer accounts sharing the failed proxy are reassigned during the same cleanup path. This closes the narrow `DeactivateProxy` route-level verification gap, while post-stream failure recovery, richer proxy/error matrix coverage on every route surface, and broader operator cutover verification still remain open.
- Landed a ninth verification slice on post-stream failure recovery policy: DB-backed ignored router tests pin the current streamed-route contract after output has already started. The gateway does not attempt same-request failover after customer-visible output has been emitted; instead it fails closed with one terminal customer-visible error boundary and persists cleanup for future requests.
- Landed a tenth verification slice to close the immediate mirror gap in that post-stream matrix: the route-level DB-backed ignored coverage now proves both post-stream `Unauthorized` and post-stream `CfBlocked` handling across both streamed surfaces, `/v1/chat/completions` and `stream=true` `/v1/responses`, including the expected account-expiry or proxy-deactivation side effects.
- Landed an eleventh verification slice on truncated post-stream behavior: DB-backed ignored router tests now prove that once partial output has already been emitted, an upstream stream that ends without a valid terminal event does not synthesize success on either streamed surface. `/v1/chat/completions` now emits the unexpected-end error payload without `[DONE]`, while `stream=true` `/v1/responses` emits `response.failed` plus `[DONE]`, and both paths persist the generic cooldown cleanup for future requests. This closes the narrow truncated/no-terminal-event route-level gap for the current policy, while deeper parser/transport interruption variants, stricter SSE ordering verification, and broader operator cutover verification still remain open.
- Landed a twelfth verification slice on post-stream parser corruption: DB-backed ignored router tests now prove that once partial output has already been emitted, a malformed SSE payload from upstream still fails closed on both streamed surfaces instead of synthesizing success or retrying the same request. `/v1/chat/completions` now emits the parser-error payload without `[DONE]`, while `stream=true` `/v1/responses` emits `response.failed` plus `[DONE]`, and both paths persist generic cooldown cleanup for future requests. This closes the narrow parser-corruption route-level gap for the current policy, while deeper transport interruption variants, stricter SSE ordering verification, and broader operator cutover verification still remain open.
- Tightened the streamed-route assertions with explicit SSE ordering checks for the main post-stream fail-closed cases on both `/v1/chat/completions` and `/v1/responses`: the route-level matrix now pins partial output before the terminal error boundary, preserves the expected terminal event shape, and avoids silently reclassifying those flows as success.
- Closed the current cutover matrix for Phase 5 acceptance scope: route-level coverage now spans `/v1/models`, `/v1/chat/completions`, `/v1/responses`, auth and plan gating, `NoAccounts`, pre-stream failover for `RateLimited`, `Unauthorized`, `CfBlocked`, and `ProxyFailed(connect refused)`, plus post-stream fail-closed handling for `Unauthorized`, `CfBlocked`, truncated/no-terminal-event, and parser corruption.
- Closed the cleanup item for the serving path: `src/services/codex_client.rs` is the active HTTP upstream executor, while `src/services/codex_oauth.rs` retains native `codex` CLI usage only for account login/bootstrap. Request-time native CLI bridge serving is no longer part of customer traffic.
- Remaining deeper transport-interruption variants are now tracked as future hardening, not as an acceptance blocker for this phase, because the current axum/reqwest harness does not reliably pin "partial output already emitted, then transport died" as a distinct route-level case.

## Success Criteria
- Operators can see why a Codex request failed without exposing secrets.
- Operators can filter recent usage by provider across current and historical rows after migration backfill runs.
- New serving path is testable and supportable.
- Temporary native-CLI request-serving logic is gone; remaining CLI usage is limited to account login/bootstrap.
- The same observability conventions can be reused when Gemini, Grok, Claude, or others enter the same gateway.

## Risk Assessment
- Observability added too late will make production issues opaque.
- Partial cleanup can leave two competing Codex serving paths.

## Security Considerations
- Logs must redact tokens, callback URLs, and sensitive provider payloads.
- Admin metrics endpoints must stay behind existing admin auth.

## Next Steps
- After cutover, reassess whether additional provider pools should be normalized into the same gateway pattern.
- Future hardening can extend the harness for deeper post-output transport-interruption cases if same-request recovery policy changes later.

Resolution:
- None for Phase 5 acceptance.
