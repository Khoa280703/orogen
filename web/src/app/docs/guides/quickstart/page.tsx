import { CodeBlock } from '@/components/code-block-with-copy';
import Link from 'next/link';

export default function QuickstartGuide() {
  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-4xl font-bold mb-4">Quickstart Guide</h1>
        <p className="text-lg text-slate-600 dark:text-slate-400">
          Get up and running with the Grok API in 5 minutes.
        </p>
      </div>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Step 1: Create an Account</h2>
        <p className="text-slate-600 dark:text-slate-400">
          Sign up using your Google account to get started quickly. No credit card required for the free tier.
        </p>
        <Link href="/login">
          <button className="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg">
            Sign up with Google
          </button>
        </Link>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Step 2: Get Your API Key</h2>
        <p className="text-slate-600 dark:text-slate-400">
          After signing in, navigate to your dashboard to generate an API key.
        </p>
        <ol className="list-decimal list-inside space-y-2 text-slate-600 dark:text-slate-400">
          <li>Go to your <Link href="/dashboard" className="text-blue-500 hover:underline">Dashboard</Link></li>
          <li>Click on &quot;API Keys&quot; in the sidebar</li>
          <li>Click &quot;Generate New Key&quot;</li>
          <li>Copy and securely store your API key</li>
        </ol>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Step 3: Make Your First API Call</h2>
        <p className="text-slate-600 dark:text-slate-400">
          Use curl to test the API:
        </p>
        <CodeBlock language="bash" title="Test with curl">
          {[
            'curl https://api.example.com/v1/chat/completions \\',
            '  -H "Authorization: Bearer your-api-key" \\',
            '  -H "Content-Type: application/json" \\',
            '  -d \'{"model": "grok-3", "messages": [{"role": "user", "content": "Hello, world!"}]}\'',
          ]}
        </CodeBlock>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Step 4: Choose Your Integration</h2>
        <p className="text-slate-600 dark:text-slate-400">
          Follow one of our language-specific guides:
        </p>
        <div className="grid md:grid-cols-2 gap-4 mt-4">
          <Link href="/docs/guides/python" className="border border-slate-200 dark:border-slate-800 rounded-lg p-4 hover:border-blue-500 transition-colors">
            <h3 className="font-semibold">Python</h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">Use with OpenAI library</p>
          </Link>
          <Link href="/docs/guides/nodejs" className="border border-slate-200 dark:border-slate-800 rounded-lg p-4 hover:border-blue-500 transition-colors">
            <h3 className="font-semibold">Node.js</h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">Build Node applications</p>
          </Link>
        </div>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Next Steps</h2>
        <ul className="space-y-2 text-slate-600 dark:text-slate-400">
          <li>
            <Link href="/docs/api" className="text-blue-500 hover:underline">
              Read the API Reference
            </Link> - Complete endpoint documentation
          </li>
          <li>
            <Link href="/docs/models" className="text-blue-500 hover:underline">
              Explore Models
            </Link> - Learn about available models
          </li>
          <li>
            <Link href="/docs/faq" className="text-blue-500 hover:underline">
              Check the FAQ
            </Link> - Common questions and answers
          </li>
        </ul>
      </section>
    </div>
  );
}
