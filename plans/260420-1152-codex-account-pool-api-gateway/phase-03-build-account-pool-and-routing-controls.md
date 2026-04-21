# Phase 03 - Build Account Pool And Routing Controls

## Context Links
- Plan overview: `plans/260420-1152-codex-account-pool-api-gateway/plan.md`
- Current pool logic: `src/account/pool.rs`
- Current retry flow: `src/api/consumer_api_support.rs`
- Account tables: `src/db/accounts.rs`, `src/db/account_credentials.rs`

## Overview
- Priority: High
- Status: Completed
- Brief: Turn the current provider pool into a scalable quota-routing layer for large multi-provider account inventory, with Codex as first hardening target, while persisting the public catalog routing model introduced logically in Phase 2.

## Key Insights
- A sellable API cannot treat all provider failures equally; rate limit, auth invalidation, proxy failure, and transient upstream errors need different routing outcomes.
- Current pool is simple round-robin with light health handling; that is not enough for “hàng vạn account”.
- Plans should limit customers; account pool should protect upstream quota and service continuity.
- Pool rules must separate provider-generic scheduling from provider-specific auth/refresh details.

## Requirements
- Support large multi-account Codex pools.
- Support the same control-plane pattern for Gemini, Grok, Claude, and future providers.
- Support account cooldown, retry budget, and health classification.
- Support future per-provider routing strategies without rewriting endpoint logic.
- Keep proxy assignment available for Codex if needed later.

## Architecture
- Extend runtime account selection with explicit account state machine:
- healthy
- cooling_down
- auth_invalid
- refresh_failed
- paused
- candidate
- Separate customer quota from upstream account health.
- Introduce provider capability metadata in routing decisions:
- auth_mode
- supports_proxy
- supports_streaming
- supports_responses_wire_api
- refresh_strategy
- Add provider-level routing metrics:
- success rate
- rate-limit streak
- auth failure streak
- last used
- cooldown until
- Split routing layers explicitly:
- public model -> route policy
- route policy -> provider candidate set
- provider candidate set -> account selector
- account selector -> executor request
- This prevents direct coupling between plans/catalog and raw provider account inventory.
- This phase owns the persisted tables and backfill strategy behind that routing boundary, not the endpoint contract itself.

## Implementation Status
- Landed explicit account routing state fields and state transitions for provider accounts.
- Landed retry/pool hardening to exhaust the full candidate set instead of single-account stickiness.
- Landed persisted public catalog routing via `public_models`, `public_model_routes`, and `plan_public_models`.
- Landed public-catalog enforcement plus admin model/plan sync so customer reads and admin writes stay on the same routing boundary.
- Landed admin observability surfaces for inspecting account health, routing state, cooldowns, and pool pressure.

## Related Code Files
- Modify: `src/account/pool.rs`
- Modify: `src/db/accounts.rs`
- Modify: `src/api/consumer_api_support.rs`
- Modify: `src/api/admin_accounts.rs`
- Create or modify: `src/db/public_models.rs`
- Create or modify: `src/db/public_model_routes.rs`
- Possible migration: `migrations/*` for cooldown/routing columns

## Implementation Steps
1. Add explicit routing state fields for accounts where current counters are insufficient.
2. Persist and backfill the public model -> upstream route mapping layer so scale does not depend on the current raw `models` table alone.
3. Refactor pool selection to skip cooling/auth-invalid accounts deterministically.
4. Teach retry layer to update account state by error type.
5. Keep provider-local rotation separate from customer plan logic.
6. Expose enough admin/debug data to inspect Codex pool pressure safely.

## Todo List
- [x] Define Codex account health state machine
- [x] Persist and backfill public model to provider route mapping
- [x] Add cooldown-aware selection
- [x] Add error-class-based account updates
- [x] Expose admin observability for pool pressure

## Success Criteria
- One bad Codex account no longer poisons the provider globally.
- Repeated 429/auth failures move accounts out of hot path automatically.
- Pool selection remains deterministic and cheap under large account counts.
- Adding Gemini or Claude later reuses the same routing/state framework.
- Plans and public catalog are no longer tightly coupled to raw upstream model records.

## Risk Assessment
- Over-aggressive cooldown can strand usable accounts.
- Under-aggressive retry can waste upstream quota and flood failing accounts.

## Validation Notes
- `cargo check` passes.
- `cargo test` passes `22/22`.
- Added unit coverage for cooldown-aware account selection, next-account start-index calculation, and strict chat content rejection.
- Later route-level DB-backed verification in Phase 5 now covers the live retry/failover paths through the real router boundary, so the current routing control slice is no longer relying on unit-only confidence.

## Security Considerations
- Admin views should expose health and metadata, never raw secrets.
- Routing state changes should be auditable.

## Next Steps
- Move Phase 4 client/docs integration onto the stabilized public boundary.

Resolution:
- None for Phase 3 acceptance.
