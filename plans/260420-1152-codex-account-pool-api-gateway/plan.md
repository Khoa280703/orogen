# Multi-Provider Account Pool API Gateway

Status: Completed
Priority: High

Context:
- Existing multi-provider model and plan enforcement already live in `duanai`.
- Current Codex path was pivoted toward native CLI bridging, which is not the right serving model for API resale.
- Reference pattern from `9router`: treat OAuth/API-key provider accounts as upstream quota/accounts, expose local compatibility APIs to customer tools.

Goal:
- Turn `duanai` into a sellable multi-provider API gateway where:
- Provider accounts are internal upstream quota sources.
- Customer API keys and plans are the public product surface.
- Public API stays stable even if upstream account capability changes.
- Codex is the first provider to harden, not a special-case architecture.

Architecture rules:
- Public product catalog must be decoupled from raw upstream provider models.
- Compatibility API handlers must normalize into one shared orchestration core.
- Provider-specific code must live behind a strict provider adapter contract.
- Plans and billing attach to public models, never directly to upstream account/model rows.
- Adding a new provider must require:
- one auth/account adapter
- one executor/translator adapter
- one capability declaration
- zero rewrites of plan enforcement or public API handler structure

Phases:
- Phase 1: [phase-01-realign-codex-provider-runtime.md](phase-01-realign-codex-provider-runtime.md) - Completed
- Phase 2: [phase-02-add-responses-compatible-api.md](phase-02-add-responses-compatible-api.md) - Completed
- Phase 3: [phase-03-build-account-pool-and-routing-controls.md](phase-03-build-account-pool-and-routing-controls.md) - Completed
- Phase 4: [phase-04-cli-and-client-integration.md](phase-04-cli-and-client-integration.md) - Completed
- Phase 5: [phase-05-observability-cutover-and-cleanup.md](phase-05-observability-cutover-and-cleanup.md) - Completed

Status note:
- Phase 1 runtime is now closed for the current architecture slice: Codex serving no longer depends on native CLI per request, provider-specific account preparation now hangs off the provider hook instead of being hard-coded inside `AccountPool`, and the current pool-routing semantics are pinned by route-level coverage rather than hidden runtime assumptions.
- Phase 2 is now wired: `/v1/chat/completions`, `/v1/responses`, and `/v1/models` all use the shared route-resolution boundary, while `responses` v1 stays text-first. Top-level tool declarations are accepted and ignored for client compatibility, but actual tool/function/computer payloads and image inputs are still rejected explicitly.
- Phase 3 has now closed the control-plane hardening loop: account routing state, cooldown-aware full-pool failover, persisted public catalog routes, `plan_public_models`, admin observability, strict chat input rejection, and admin model/plan sync all land on the same public routing boundary.
- Phase 4 is now closed on the current customer surface: docs describe public-catalog model discovery, OpenAI-compatible usage, Codex CLI setup through the gateway’s `responses` surface, a provider-onboarding checklist for future adapters, and an admin-side config snippet helper that can use plan-filtered `/v1/models`. A DB-backed ignored smoke test now also proves the local `codex` CLI binary can complete a simple non-interactive prompt through the gateway end to end.
- Phase 5 closes the observability and cutover loop: usage logs expose provider-aware admin visibility, docs reflect the current `/v1/responses` compatibility contract, and the route-level verification matrix now pins the serving path through auth, plan, failover, and fail-closed behavior.
- Phase 5 verification now spans the full current public contract: `/v1/models`, `/v1/chat/completions`, and `/v1/responses` all have DB-backed route-level coverage for auth, plan enforcement, request-shape rejection, `NoAccounts`, pre-stream failover, and post-stream fail-closed semantics through the real router boundary.
- Phase 5 now also exposes provider onboarding verification gates in admin health so operators can spot provider rollout signals without adapters, auth-mode mismatches, routed public models with no active selling plans, or route-without-selectable-account states before a provider rollout is considered healthy.
- Phase 5 now also updates the support/docs surface for rollout: `/docs/api` documents the public `/v1/responses` contract, stream failure semantics, production error payload shape, auth bootstrap behavior, and current 400/403/503 boundaries, while `/docs/guides/provider-onboarding` now includes the required backend verification commands, admin-health acceptance criteria, rollback/support flow, and an explicit warning that router-level checks do not replace adapter/auth or retry/failover validation.
- Phase 5 now also adds local-upstream failover verification: DB-backed ignored router tests spin a stub Codex upstream plus a seeded two-account pool to prove through the real router-to-local-stub path that pre-stream rate-limit and unauthorized/auth-expiry failures on the first account fall through to a healthy second account for both `/v1/chat/completions` and streamed `/v1/responses`, while persisting the expected cooldown or auth-invalid/session-expired state on the failed account. It also now covers `DeactivateProxy` at route level through the same DB-backed harness: proxy-classified pre-stream failures, including both `CfBlocked` and transport-level `ProxyFailed(connect refused)`, deactivate the failed proxy persistently, reassign the affected account cohort onto a healthy replacement proxy, and recover the same request for both `/v1/chat/completions` and streamed `/v1/responses` before any customer-visible failure payload is emitted. This is still routing-state/fallback verification, not full upstream request-contract verification, and it still depends on `DATABASE_URL` plus `CREATE/DROP DATABASE` access outside the default `cargo test` path.
- Phase 5 now closes its current acceptance scope: DB-backed ignored router tests and helper coverage together pin `/v1/models`, `/v1/chat/completions`, and `/v1/responses` across auth/plan gating, request-shape rejection, `NoAccounts`, pre-stream failover for `RateLimited`, `Unauthorized`, `CfBlocked`, and `ProxyFailed(connect refused)`, plus post-stream fail-closed handling for `Unauthorized`, `CfBlocked`, truncated/no-terminal-event, and parser corruption. The streamed-route matrix now also includes stricter SSE ordering assertions for the main post-stream error cases. Request-time native CLI bridge serving is gone; `src/services/codex_oauth.rs` keeps `codex` CLI only for account login/bootstrap. Deeper transport interruption variants remain future hardening because the current harness does not reliably isolate "partial output emitted, then transport died" as a distinct route-level case.

Execution note:
- Phase 2 owns the shared orchestration contract and public-model resolution interface.
- Phase 3 owns persisted routing state, catalog backfill, and large-pool scheduling hardening.
- Phase 4 must consume the catalog/routing rules from Phase 3, not redefine product mapping in UI/docs code.

Current code anchors:
- Account pool: `src/account/pool.rs`
- Codex auth lifecycle: `src/services/codex_oauth.rs`
- Chat surface: `src/api/chat_completions.rs`
- Consumer chat: `src/api/consumer_chat.rs`
- Plan enforcement: `src/api/plan_enforcement.rs`
- Model registry: `src/db/models.rs`, `src/api/models.rs`

Target core modules:
- Provider account control plane
- public providers / provider capabilities
- provider accounts / credentials / health
- Public product catalog
- public models sold to customers
- upstream model mappings per provider
- Compatibility API layer
- `/v1/chat/completions`
- `/v1/responses`
- future additional compatibility surfaces
- Shared orchestration core
- request normalization
- model resolution
- account selection
- provider execution
- stream/result normalization
- Product enforcement layer
- API keys
- plans
- quotas
- usage logging

Target architecture:
- Upstream serving uses provider executors behind a provider-agnostic contract.
- Each provider may use a different auth strategy:
- OAuth token accounts
- API key accounts
- cookie/session accounts
- future custom enterprise connectors
- Public API exposes at least `/v1/models`, `/v1/chat/completions`, `/v1/responses`.
- Customer plans gate public models that `duanai` chooses to sell, not raw upstream catalog 1:1.
- Routing chooses healthy provider accounts from internal pools with refresh, cooldown, failover, proxy support, and usage accounting.
- Provider-specific logic stays inside adapters/executors, not in plan logic or public API handlers.
- Client compatibility is a separate layer from provider routing:
- public OpenAI-compatible endpoints
- public Responses-compatible endpoints
- future Anthropic-compatible or provider-specific compatibility shims if needed

Non-goals:
- Exposing real upstream account credentials to customers.
- Binding serving correctness to local native Codex CLI installation.
- Mirroring every upstream model variant immediately.

Dependencies:
- Existing providers/models/plan_models schema
- Existing account + account_credentials schema
- Existing admin accounts UI and plans UI

Schema direction:
- Keep existing `providers`, `models`, `plan_models` as current baseline only.
- Introduce a clean split if needed:
- `provider_capabilities` or equivalent metadata source
- `public_models` for sellable catalog
- `public_model_routes` or equivalent mapping from public model to provider/upstream model
- Plans should reference `public_models`, not raw upstream-only rows, once migration completes.

Success criteria:
- A customer API key can call a stable compatibility API without knowing internal account details.
- A Codex CLI user can point config/env at `duanai` and work through the gateway.
- New providers such as Gemini, Grok, Claude, Qwen can plug into the same routing/product model without rewriting the platform core.
- Internal account failures degrade through pool fallback instead of disabling the whole provider.
- Plans remain the single source of truth for sellable quota and model access.

Risks:
- Upstream provider contracts may drift independently.
- Large account pools need explicit cooldown, routing fairness, and auth refresh strategy.
- Raw upstream model catalog can outpace what should be sold publicly.

Cook handoff:
- `$ cook --auto /home/khoa2807/working-sources/duanai/plans/260420-1152-codex-account-pool-api-gateway`

Resolution:
- None for this plan scope.
