import Link from 'next/link';

import { CodeBlock } from '@/components/code-block-with-copy';

export default function ProviderOnboardingGuide() {
  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-4xl font-bold mb-4">Provider Onboarding</h1>
        <p className="text-lg text-slate-600 dark:text-slate-400">
          Internal checklist for adding a new upstream provider without rewriting plans, public APIs, or customer-facing model slugs.
        </p>
      </div>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Architecture Rules</h2>
        <div className="grid md:grid-cols-3 gap-4">
          <div className="border border-slate-200 dark:border-slate-800 rounded-lg p-4">
            <h3 className="font-semibold mb-2">Public Catalog First</h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">Plans and customer API keys must attach to public model slugs, never directly to provider-owned raw models.</p>
          </div>
          <div className="border border-slate-200 dark:border-slate-800 rounded-lg p-4">
            <h3 className="font-semibold mb-2">Adapter Boundary</h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">Provider auth, execution, and capability checks stay behind provider adapters instead of leaking into API handlers.</p>
          </div>
          <div className="border border-slate-200 dark:border-slate-800 rounded-lg p-4">
            <h3 className="font-semibold mb-2">Stable Customer Surface</h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">Client integrations should keep using the same base URL, API keys, and public slugs even if internal account routing changes.</p>
          </div>
        </div>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Checklist</h2>
        <ol className="list-decimal list-inside space-y-2 text-slate-600 dark:text-slate-400">
          <li>Add the provider record and upstream models in the backend control plane.</li>
          <li>Implement the provider auth/account adapter so pooled accounts can refresh, cool down, and fail over safely.</li>
          <li>Implement the executor/translator adapter behind the shared orchestration boundary.</li>
          <li>Declare capabilities explicitly so unsupported surfaces are rejected early instead of partially routed.</li>
          <li>Create or sync public catalog entries and map them through <code>public_model_routes</code>.</li>
          <li>Attach plans to public models, not raw upstream model rows.</li>
          <li>Verify <code>GET /v1/models</code>, <code>POST /v1/chat/completions</code>, and <code>POST /v1/responses</code> against the new route.</li>
          <li>Document the client profile clearly, including any text-first, no-tools, or tool-declarations-ignored limitations.</li>
        </ol>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Rollout Verification</h2>
        <p className="text-slate-600 dark:text-slate-400">
          Before you call a provider rollout healthy, run the default backend suite and the ignored DB-backed router suite together. The ignored suite proves the public compatibility routes still pass through auth, plan enforcement, and SSE boundaries without touching upstream, but it is only one gate in the rollout checklist.
        </p>
        <CodeBlock language="bash" title="Backend verification commands">
          {[
            'cargo check',
            'cargo test -- --nocapture',
            'cargo test api::tests:: -- --ignored --nocapture',
          ]}
        </CodeBlock>
        <p className="text-sm text-slate-600 dark:text-slate-400">
          The ignored router tests require <code>DATABASE_URL</code> plus PostgreSQL permissions for <code>CREATE DATABASE</code> and <code>DROP DATABASE</code>. Do not mark provider onboarding complete if that suite has not been run in a suitable environment.
        </p>
        <p className="text-sm text-slate-600 dark:text-slate-400">
          Verify both sides of the streaming policy: pre-stream failures should recover or fail over before customer-visible bytes are emitted, while post-stream failures must fail closed with the documented terminal event shape instead of silently switching accounts mid-stream.
        </p>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Admin Health Acceptance</h2>
        <p className="text-slate-600 dark:text-slate-400">
          Check <code>/admin/health</code> before and after rollout. The provider is only ready for customer traffic when the verification gate is clean, not merely when the router tests pass.
        </p>
        <ul className="space-y-2 text-slate-600 dark:text-slate-400">
          <li><code>ready</code> is <code>true</code> and <code>warnings</code> is empty for the provider gate.</li>
          <li><code>has_chat_adapter</code> is <code>true</code>, and the registered adapter matches the provider you expect to serve.</li>
          <li><code>expected_auth_mode</code> matches the active accounts you onboarded for that provider.</li>
          <li>If <code>active_public_route_count</code> is greater than zero, <code>selectable_account_count</code> must also be greater than zero.</li>
          <li>If you intend to sell the route immediately, <code>plan_assignment_count</code> must be greater than zero.</li>
          <li>Run at least one adapter/auth verification pass and one retry or failover drill outside the router-only suite before calling rollout healthy.</li>
        </ul>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Implementation Anchors</h2>
        <ul className="space-y-2 text-slate-600 dark:text-slate-400">
          <li><code>src/account/</code> owns pooled-account selection, cooldown, and failover.</li>
          <li><code>src/providers/</code> owns provider-specific auth and execution adapters.</li>
          <li><code>src/api/request_orchestrator.rs</code> owns model resolution and shared request normalization.</li>
          <li><code>src/db/public_models.rs</code> and <code>src/db/public_model_routes.rs</code> own the sellable public catalog boundary.</li>
          <li><code>src/api/models.rs</code> is the customer-visible discovery surface and must stay aligned with plan visibility.</li>
        </ul>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Cutover Rules</h2>
        <div className="space-y-3 text-slate-600 dark:text-slate-400">
          <p>Do not expose internal provider credentials, browser sessions, or OAuth state to customers.</p>
          <p>Do not ship a provider by mirroring every upstream model automatically. Curate public slugs intentionally and route them through the catalog layer.</p>
          <p>Do not claim tool, image, or agentic support in docs until the shared compatibility surface accepts those payloads end to end.</p>
          <p>Do not treat router-level verification as failover verification. Retry/failover behavior and operator cutover checks still need their own validation path.</p>
        </div>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Rollback And Support Flow</h2>
        <ol className="list-decimal list-inside space-y-2 text-slate-600 dark:text-slate-400">
          <li>If provider warnings appear in <code>/admin/health</code> or <code>selectable_account_count</code> drops to zero, stop selling the affected route first by removing its plan assignment or disabling the backing public route in the control plane.</li>
          <li>Capture the provider gate state from <code>/admin/health</code> and the recent provider-filtered request history from <code>/admin/usage</code> before rotating credentials or proxies.</li>
          <li>Re-check account auth state, proxy health, and any adapter-specific login or refresh flow before reopening traffic.</li>
          <li>Do not re-enable the route until the provider gate returns to <code>ready: true</code> with no warnings and the failover drill succeeds again.</li>
        </ol>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Related Docs</h2>
        <ul className="space-y-2 text-slate-600 dark:text-slate-400">
          <li>
            <Link href="/docs/models" className="text-blue-500 hover:underline">
              Models
            </Link> - Public catalog and plan visibility rules
          </li>
          <li>
            <Link href="/docs/api" className="text-blue-500 hover:underline">
              API Reference
            </Link> - Current compatibility contract
          </li>
          <li>
            <Link href="/docs/guides/codex-cli" className="text-blue-500 hover:underline">
              Codex CLI
            </Link> - Example of a client profile constrained to the current text-first path
          </li>
        </ul>
      </section>
    </div>
  );
}
