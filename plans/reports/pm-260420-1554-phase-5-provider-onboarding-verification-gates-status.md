## Scope
- Plan: `plans/260420-1152-codex-account-pool-api-gateway`
- Phase: `phase-05-observability-cutover-and-cleanup.md`
- Slice: provider onboarding verification gates

## Status call
- Phase 5 stays `In Progress`.
- Verification-gates slice can count complete as a slice.
- Do not finalize Phase 5 from this tick alone.

## Why
- Plan still marks Phase 5 `In Progress` in overview and phase file.
- Phase 5 todo still has 3 open items:
- `[ ] Verify cutover matrix end-to-end`
- `[ ] Remove obsolete bridge logic`
- `[ ] Update support/deployment docs`
- Phase file explicitly says current verification coverage is helper/unit only, not endpoint-level or end-to-end account-routing validation.

## Evidence
- Plan overview notes Phase 5 verification gates landed but broader matrix still open: `plan.md`
- Phase 5 todo marks verification gates done, other items open: `phase-05-observability-cutover-and-cleanup.md`
- Backend health payload now exposes provider verification gate fields and warnings logic: `src/api/admin_health.rs`
- Admin health UI renders provider verification gate cards: `web/src/app/admin/health/page.tsx`
- Unit tests pass for missing-accounts, auth-mode mismatch, ready path:
- `cargo test flags_missing_accounts_for_routed_provider -- --nocapture`
- `cargo test flags_auth_mode_mismatch -- --nocapture`
- `cargo test marks_provider_ready_when_gates_are_clear -- --nocapture`

## Slice completeness
- Complete for planned slice scope:
- operator-visible onboarding readiness signals exist
- warnings cover missing adapter, route-without-account, route-without-plan, auth-mode mismatch
- backend has unit coverage
- admin UI exposes the result
- Not complete for wider Phase 5 verification/cutover scope:
- no endpoint/e2e cutover matrix close
- no obsolete bridge cleanup close
- no support/deployment doc close

## Docs impact
- Minor, still pending.
- No obvious customer-doc blocker for this slice.
- Operator/support docs should mention the new admin health verification-gates panel before Phase 5 finalization.

## Recommended plan state
- Keep Phase 5 `In Progress`.
- Keep provider onboarding verification gates todo checked.
- Treat this as a completed sub-slice under Phase 5, not a phase-finalization trigger.

## Open items
- End-to-end cutover verification matrix across `/v1/models`, `/v1/chat/completions`, `/v1/responses`, plan enforcement, account fallback
- Remove or quarantine obsolete Codex bridge/native CLI logic
- Update support/deployment/operator docs
- Optional hardening: add endpoint/integration coverage for admin health payload itself

## Unresolved questions
- Should operator docs live in a new root `docs/` tree or under existing `web` docs surface for this repo?
