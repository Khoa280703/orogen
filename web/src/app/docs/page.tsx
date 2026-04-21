import Link from 'next/link';
import { ArrowRight } from 'lucide-react';
import { Button } from '@/components/ui/button';

export default function DocsHome() {
  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-4xl font-bold mb-4">Welcome to the API Docs</h1>
        <p className="text-lg text-slate-600 dark:text-slate-400">
          Build against a stable multi-provider API surface. Start with your API key, your base URL, and a model returned by <code>/v1/models</code>.
        </p>
      </div>

      <div className="grid md:grid-cols-2 gap-6">
        <Link href="/docs/guides/quickstart">
          <div className="border border-slate-200 dark:border-slate-800 rounded-lg p-6 hover:border-blue-500 transition-colors">
            <h3 className="text-xl font-semibold mb-2">Quickstart Guide</h3>
            <p className="text-slate-600 dark:text-slate-400 mb-4">
              Get up and running in 5 minutes with our quickstart tutorial.
            </p>
            <Button>
              Start now <ArrowRight className="ml-2 h-4 w-4" />
            </Button>
          </div>
        </Link>

        <Link href="/docs/api">
          <div className="border border-slate-200 dark:border-slate-800 rounded-lg p-6 hover:border-blue-500 transition-colors">
            <h3 className="text-xl font-semibold mb-2">API Reference</h3>
            <p className="text-slate-600 dark:text-slate-400 mb-4">
              Complete API documentation with examples and error codes.
            </p>
            <Button variant="outline">
              View API <ArrowRight className="ml-2 h-4 w-4" />
            </Button>
          </div>
        </Link>
      </div>

      <div>
        <h2 className="text-2xl font-semibold mb-4">Key Features</h2>
        <div className="grid md:grid-cols-3 gap-4">
          <div className="border border-slate-200 dark:border-slate-800 rounded-lg p-4">
            <h3 className="font-semibold mb-2">Multiple Models</h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">
              Sellable models come from the public catalog, not raw upstream inventory.
            </p>
          </div>
          <div className="border border-slate-200 dark:border-slate-800 rounded-lg p-4">
            <h3 className="font-semibold mb-2">Streaming Support</h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">
              Real-time token streaming for better UX.
            </p>
          </div>
          <div className="border border-slate-200 dark:border-slate-800 rounded-lg p-4">
            <h3 className="font-semibold mb-2">Client Compatibility</h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">
              OpenAI-compatible SDKs and the current text-first Codex CLI path can point at the same gateway.
            </p>
          </div>
        </div>
      </div>

      <div>
        <h2 className="text-2xl font-semibold mb-4">Popular Guides</h2>
        <div className="space-y-2">
          <Link href="/docs/guides/codex-cli" className="block p-4 border border-slate-200 dark:border-slate-800 rounded-lg hover:border-blue-500 transition-colors">
            <h3 className="font-semibold">Codex CLI</h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">Point Codex CLI at this gateway for the current text-first Responses path</p>
          </Link>
          <Link href="/docs/guides/provider-onboarding" className="block p-4 border border-slate-200 dark:border-slate-800 rounded-lg hover:border-blue-500 transition-colors">
            <h3 className="font-semibold">Provider Onboarding</h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">Internal checklist for adding Gemini, Claude, Grok, or other providers cleanly</p>
          </Link>
          <Link href="/docs/guides/python" className="block p-4 border border-slate-200 dark:border-slate-800 rounded-lg hover:border-blue-500 transition-colors">
            <h3 className="font-semibold">Python Integration</h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">Use the OpenAI library with our API</p>
          </Link>
          <Link href="/docs/guides/nodejs" className="block p-4 border border-slate-200 dark:border-slate-800 rounded-lg hover:border-blue-500 transition-colors">
            <h3 className="font-semibold">Node.js Integration</h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">Build Node.js applications with our API</p>
          </Link>
          <Link href="/docs/guides/langchain" className="block p-4 border border-slate-200 dark:border-slate-800 rounded-lg hover:border-blue-500 transition-colors">
            <h3 className="font-semibold">LangChain Integration</h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">Powerful LLM orchestration with LangChain</p>
          </Link>
        </div>
      </div>
    </div>
  );
}
