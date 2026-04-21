import Link from 'next/link';

import { CodeBlock } from '@/components/code-block-with-copy';

export default function CodexCliGuide() {
  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-4xl font-bold mb-4">Codex CLI</h1>
        <p className="text-lg text-slate-600 dark:text-slate-400">
          Point Codex CLI at this gateway so the CLI uses your customer API key and the gateway handles upstream account routing behind the scenes for basic text-first Responses flows.
        </p>
      </div>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Before You Start</h2>
        <ol className="list-decimal list-inside space-y-2 text-slate-600 dark:text-slate-400">
          <li>Create a customer API key from your dashboard.</li>
          <li>Pick a model slug returned by <code>GET /v1/models</code>.</li>
          <li>Point Codex CLI at this gateway, not at the upstream provider directly.</li>
        </ol>
        <CodeBlock language="bash" title="Discover plan-visible models">
          {[
            'curl https://your-duanai-domain.com/v1/models \\',
            '  -H "Authorization: Bearer your-api-key"',
          ]}
        </CodeBlock>
        <p className="text-sm text-slate-600 dark:text-slate-400">
          Use the API key if you want the list filtered by the customer plan. Anonymous callers can still inspect the public catalog.
        </p>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Option 1: Environment Variables</h2>
        <p className="text-slate-600 dark:text-slate-400">
          This is the quickest setup if you only need Codex CLI to use the gateway for the current shell session on text-first prompt flows.
        </p>
        <CodeBlock language="bash" title="Shell setup">
          {[
            'export OPENAI_BASE_URL="https://your-duanai-domain.com/v1"',
            'export OPENAI_API_KEY="your-api-key"',
            '',
            'codex --model your-model-id "Explain this repository structure."',
          ]}
        </CodeBlock>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Option 2: Persistent Codex Config</h2>
        <p className="text-slate-600 dark:text-slate-400">
          If you want Codex CLI to keep using the gateway by default, write a provider entry in <code>~/.codex/config.toml</code>.
        </p>
        <CodeBlock language="toml" title="~/.codex/config.toml">
          {[
            'model = "your-model-id"',
            'model_provider = "duanai"',
            '',
            '[model_providers.duanai]',
            'name = "DuanAI"',
            'base_url = "https://your-duanai-domain.com/v1"',
            'wire_api = "responses"',
            '',
            '[agents.subagent]',
            'model = "your-model-id"',
          ]}
        </CodeBlock>
        <p className="text-sm text-slate-600 dark:text-slate-400">
          Keep both model fields on a slug returned by <code>GET /v1/models</code>. Do not assume specific Codex model IDs are always present on every deploy.
        </p>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Authentication File</h2>
        <p className="text-slate-600 dark:text-slate-400">
          Codex CLI can also read the API key from <code>~/.codex/auth.json</code>.
        </p>
        <CodeBlock language="json" title="~/.codex/auth.json">
          {[
            '{',
            '  "OPENAI_API_KEY": "your-api-key"',
            '}',
          ]}
        </CodeBlock>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Why This Works</h2>
        <div className="grid md:grid-cols-3 gap-4">
          <div className="border border-slate-200 dark:border-slate-800 rounded-lg p-4">
            <h3 className="font-semibold mb-2">Public Catalog</h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">Codex CLI sees only public model slugs sold by your plan.</p>
          </div>
          <div className="border border-slate-200 dark:border-slate-800 rounded-lg p-4">
            <h3 className="font-semibold mb-2">Text-Only Responses</h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">The current gateway path is suitable for text-first Responses requests. Codex CLI tool declarations are accepted for compatibility, but they are ignored unless the gateway gains native tool execution later.</p>
          </div>
          <div className="border border-slate-200 dark:border-slate-800 rounded-lg p-4">
            <h3 className="font-semibold mb-2">No Upstream Login</h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">Customers use their own gateway API key instead of owning internal provider accounts.</p>
          </div>
        </div>
        <p className="text-sm text-slate-600 dark:text-slate-400">
          Current limitation: simple Codex CLI prompts now work through the gateway, but richer agentic Responses flows still stop at the text path. If a workflow depends on actual function/computer call execution or image inputs, the gateway will still reject that request shape today.
        </p>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Related Guides</h2>
        <ul className="space-y-2 text-slate-600 dark:text-slate-400">
          <li>
            <Link href="/docs/api" className="text-blue-500 hover:underline">
              API Reference
            </Link> - Inspect <code>/v1/responses</code> and <code>/v1/models</code>
          </li>
          <li>
            <Link href="/docs/models" className="text-blue-500 hover:underline">
              Models
            </Link> - Learn how plan-visible model discovery works
          </li>
          <li>
            <Link href="/docs/guides/quickstart" className="text-blue-500 hover:underline">
              Quickstart
            </Link> - Basic OpenAI-compatible setup if you do not need Codex CLI
          </li>
        </ul>
      </section>
    </div>
  );
}
