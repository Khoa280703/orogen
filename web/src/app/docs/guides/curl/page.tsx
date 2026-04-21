import Link from 'next/link';

import { CodeBlock } from '@/components/code-block-with-copy';

export default function CurlGuide() {
  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-4xl font-bold mb-4">cURL Guide</h1>
        <p className="text-lg text-slate-600 dark:text-slate-400">
          Use cURL to interact with the gateway directly from your terminal.
        </p>
      </div>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Basic Request</h2>
        <CodeBlock language="bash">
          {[
            'curl https://your-duanai-domain.com/v1/chat/completions \\',
            '  -H "Authorization: Bearer your-api-key" \\',
            '  -H "Content-Type: application/json" \\',
            '  -d \'{"model": "your-model-id", "messages": [{"role": "user", "content": "Hello, world!"}]}\'',
          ]}
        </CodeBlock>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">With System Prompt</h2>
        <CodeBlock language="bash">
          {[
            'curl https://your-duanai-domain.com/v1/chat/completions \\',
            '  -H "Authorization: Bearer your-api-key" \\',
            '  -H "Content-Type: application/json" \\',
            '  -d \'{"model": "your-model-id", "messages": [{"role": "system", "content": "You are a helpful coding assistant."}, {"role": "user", "content": "Write a Python function."}]}\'',
          ]}
        </CodeBlock>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">List Available Models</h2>
        <CodeBlock language="bash">
          {[
            'curl https://your-duanai-domain.com/v1/models \\',
            '  -H "Authorization: Bearer your-api-key"',
          ]}
        </CodeBlock>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Using Environment Variable</h2>
        <CodeBlock language="bash">
          {[
            '# Set your API key',
            'export DUANAI_API_KEY="your-api-key"',
            '',
            '# Use in curl',
            'curl https://your-duanai-domain.com/v1/chat/completions \\',
            '  -H "Authorization: Bearer $DUANAI_API_KEY" \\',
            '  -H "Content-Type: application/json" \\',
            '  -d \'{"model": "your-model-id", "messages": [{"role": "user", "content": "Hello!"}]}\'',
          ]}
        </CodeBlock>
        <p className="text-sm text-slate-600 dark:text-slate-400">
          For Codex CLI specifically, use the dedicated <Link href="/docs/guides/codex-cli" className="text-blue-500 hover:underline">Codex CLI guide</Link>.
        </p>
      </section>
    </div>
  );
}
