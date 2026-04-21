# Phase 04 - CLI And Client Integration

## Context Links
- Plan overview: `plans/260420-1152-codex-account-pool-api-gateway/plan.md`
- Admin/web client code: `web/src/app/admin/accounts/page.tsx`, `web/src/app/admin/plans/plan-form-dialog.tsx`
- 9router CLI integration: `9router/src/app/api/cli-tools/codex-settings/route.js`
- 9router docs: `9router/i18n/README.vi.md`

## Overview
- Priority: High
- Status: Completed
- Brief: Make customer-facing integration simple: API key + model + endpoint, including Codex CLI-specific config.

## Key Insights
- Customers should not think in terms of upstream accounts.
- Codex CLI can be pointed to a custom model provider / base URL instead of owning the upstream account itself.
- Selling API means installation/config UX matters nearly as much as core serving.
- A multi-provider platform must separate “client integration type” from “upstream provider type”.

## Requirements
- Support plain OpenAI-compatible clients.
- Support Codex CLI setup guidance or generated copy-paste config snippets.
- Keep plan-specific model visibility clean and intentional.
- Leave room for other client families later:
- Claude Code / Anthropic-style
- Gemini CLI
- IDE plugins that expect OpenAI-compatible endpoints

## Architecture
- Add a CLI integration helper surface:
- generated env snippets
- generated config snippets
- optional “apply config locally” admin/helper action later
- Consume the curated public model catalog defined by backend routing layers; do not redefine model mapping in UI/docs.
- Keep internal upstream account models decoupled from public catalog.
- Treat client tools as compatibility profiles:
- OpenAI-compatible
- Responses-compatible
- Anthropic-compatible
- provider-native helper profiles when necessary

## Related Code Files
- Modify: `web/src/lib/user-api.ts`
- Modify: `web/src/app/docs/*`
- Modify: `web/src/app/admin/*` where helpful
- Possible backend additions: `src/api/*` for CLI config helper endpoints

## Implementation Status
- Landed docs updates that shift public messaging from raw Grok examples to a public-catalog gateway surface.
- Landed a dedicated Codex CLI guide with env-var setup, `~/.codex/config.toml`, and `~/.codex/auth.json` examples pointing at `duanai` for the current text-first path.
- Landed docs updates for `/v1/models` and `/v1/responses` so customer integrations are framed around plan-visible public slugs, anonymous-vs-plan-filtered catalog discovery, and current Responses compatibility limits.
- Landed frontend changes that stop falling back to hard-coded Grok model catalogs when `/v1/models` is unavailable.
- Landed a provider onboarding guide so future Gemini/Grok/Claude additions follow the same public-catalog, adapter, and plan-assignment rules.
- Removed the temporary Codex display-name rewrite during CCS cache sync so catalog metadata now preserves upstream naming and leaves UI label decisions to the client surface.
- Landed admin-side Codex CLI config helper snippets that consume the public catalog, switch to plan-filtered `/v1/models` when a customer API key is provided, and generate `OPENAI_*`, `~/.codex/config.toml`, and `~/.codex/auth.json` examples without exposing internal provider credentials.
- Landed a DB-backed ignored smoke test that runs the local `codex` CLI binary non-interactively against the gateway’s `/v1/responses` surface with a seeded public model, customer API key, and stub Codex upstream. The current compatibility path now proves simple Codex CLI prompts complete end-to-end through the gateway.
- Relaxed the `/v1/responses` request gate so top-level `tools` declarations are accepted and ignored for client compatibility. Actual tool/function/computer payloads in the request body are still rejected until the gateway gains native tool execution.

## Implementation Steps
1. Consume and present the public sellable models and aliases defined by backend catalog/routing, starting with Codex-backed offerings.
2. Update `/v1/models` output rules and docs/UI rendering to reflect sellable catalog, not raw upstream everything.
3. Add documentation/UI snippets for:
- OpenAI-compatible SDKs
- Codex CLI
- Claude Code if kept compatible
4. Keep plan-model assignment based on public catalog entries.
5. Document provider onboarding rules so adding Gemini/Grok/Claude follows the same catalog + route pattern.

## Todo List
- [x] Define public sellable catalog
- [x] Consume backend-defined public catalog consistently
- [x] Align `/v1/models` with plan visibility
- [x] Add Codex CLI configuration docs/snippets
- [x] Add provider onboarding checklist
- [x] Remove temporary model alias hacks once public catalog exists
- [x] Run real Codex CLI end-to-end validation on the current compatibility path

## Success Criteria
- A customer can integrate with `duanai` without learning internal Codex account mechanics.
- Models shown in UI/docs match what plans actually sell.
- Codex CLI setup for the currently supported compatibility path is a config exercise, not an account-login exercise.
- The same product surface can later sell Gemini, Grok, Claude capacity through the same API/product layer.

## Risk Assessment
- If public catalog mirrors upstream too closely, product pricing and support burden become unstable.
- If public catalog is too abstract, power users may feel constrained.

## Security Considerations
- Generated snippets must only reference customer API keys, never internal provider credentials.
- Local config helper features must avoid overwriting unrelated user config unexpectedly.

## Validation Notes
- `npm run build` passes in `web/`.
- Scoped `npx eslint` for the touched files reports 0 errors and 3 pre-existing hook dependency warnings in `web/src/app/admin/accounts/page.tsx`.
- Full `npm run lint` still reports pre-existing issues outside this scope in login, chat, admin, dashboard, and image components.
- `cargo check` passes after the backend catalog-sync cleanup in `src/db/migrate.rs`.
- `cargo test codex_exec_can_use_gateway_responses_surface_end_to_end -- --ignored --nocapture` now passes with the local `codex` binary.

## Next Steps
- Revisit richer agentic Codex CLI flows only after the gateway can execute or translate actual tool/function/computer call payloads instead of ignoring top-level tool declarations.

Resolution:
- None for Phase 4 acceptance.
