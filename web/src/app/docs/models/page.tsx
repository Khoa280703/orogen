import { CodeBlock } from '@/components/code-block-with-copy';

export default function ModelsPage() {
  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-4xl font-bold mb-4">Available Models</h1>
        <p className="text-lg text-slate-600 dark:text-slate-400">
          Treat the public catalog as the source of truth. The model slugs available to your API key come from <code>GET /v1/models</code>, not directly from upstream providers.
        </p>
      </div>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">How To Discover Models</h2>
        <p className="text-slate-600 dark:text-slate-400">
          Query the catalog anonymously to inspect the public catalog, or use the same customer API key your application will use in production to get a plan-filtered view.
        </p>
        <CodeBlock language="bash">
          {[
            'curl https://your-duanai-domain.com/v1/models \\',
            '  -H "Authorization: Bearer your-api-key"',
          ]}
        </CodeBlock>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Example Response</h2>
        <CodeBlock language="json">
          {`{
  "object": "list",
  "data": [
    {
      "id": "your-chat-model",
      "object": "model",
      "owned_by": "codex"
    },
    {
      "id": "your-image-or-alt-model",
      "object": "model",
      "owned_by": "grok"
    }
  ]
}`}
        </CodeBlock>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Catalog Rules</h2>
        <div className="grid md:grid-cols-3 gap-4">
          <div className="border border-slate-200 dark:border-slate-800 rounded-lg p-4">
            <h3 className="font-semibold mb-2">Plan-Visible</h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">Only models sold by your plan appear in the response.</p>
          </div>
          <div className="border border-slate-200 dark:border-slate-800 rounded-lg p-4">
            <h3 className="font-semibold mb-2">Public Slugs</h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">Use public slugs from this list instead of relying on raw upstream names.</p>
          </div>
          <div className="border border-slate-200 dark:border-slate-800 rounded-lg p-4">
            <h3 className="font-semibold mb-2">Route-Stable</h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">Backend routing may change internally without forcing client-side config changes.</p>
          </div>
        </div>
      </section>
    </div>
  );
}
