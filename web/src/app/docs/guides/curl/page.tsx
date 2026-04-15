import { CodeBlock } from '@/components/code-block-with-copy';

export default function CurlGuide() {
  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-4xl font-bold mb-4">cURL Guide</h1>
        <p className="text-lg text-slate-600 dark:text-slate-400">
          Use cURL to interact with the Grok API directly from your terminal.
        </p>
      </div>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Basic Request</h2>
        <CodeBlock language="bash">
          {[
            'curl https://api.example.com/v1/chat/completions \\',
            '  -H "Authorization: Bearer your-api-key" \\',
            '  -H "Content-Type: application/json" \\',
            '  -d \'{"model": "grok-3", "messages": [{"role": "user", "content": "Hello, world!"}]}\'',
          ]}
        </CodeBlock>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">With System Prompt</h2>
        <CodeBlock language="bash">
          {[
            'curl https://api.example.com/v1/chat/completions \\',
            '  -H "Authorization: Bearer your-api-key" \\',
            '  -H "Content-Type: application/json" \\',
            '  -d \'{"model": "grok-3", "messages": [{"role": "system", "content": "You are a helpful coding assistant."}, {"role": "user", "content": "Write a Python function."}]}\'',
          ]}
        </CodeBlock>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">List Available Models</h2>
        <CodeBlock language="bash">
          {[
            'curl https://api.example.com/v1/models \\',
            '  -H "Authorization: Bearer your-api-key"',
          ]}
        </CodeBlock>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Using Environment Variable</h2>
        <CodeBlock language="bash">
          {[
            '# Set your API key',
            'export GROK_API_KEY="your-api-key"',
            '',
            '# Use in curl',
            'curl https://api.example.com/v1/chat/completions \\',
            '  -H "Authorization: Bearer $GROK_API_KEY" \\',
            '  -H "Content-Type: application/json" \\',
            '  -d \'{"model": "grok-3", "messages": [{"role": "user", "content": "Hello!"}]}\'',
          ]}
        </CodeBlock>
      </section>
    </div>
  );
}
