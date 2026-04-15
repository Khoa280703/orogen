import { CodeBlock } from '@/components/code-block-with-copy';

export default function ModelsPage() {
  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-4xl font-bold mb-4">Available Models</h1>
        <p className="text-lg text-slate-600 dark:text-slate-400">
          Use exact model slugs from the platform. Aliases like <code>grok-latest</code> are no longer supported.
        </p>
      </div>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Grok-3</h2>
        <p className="text-slate-600 dark:text-slate-400">
          Our flagship model with excellent general-purpose capabilities. Perfect for chatbots, content generation, and code assistance.
        </p>
        <div className="grid md:grid-cols-3 gap-4">
          <div className="border border-slate-200 dark:border-slate-800 rounded-lg p-4">
            <h3 className="font-semibold mb-2">Context Length</h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">128K tokens</p>
          </div>
          <div className="border border-slate-200 dark:border-slate-800 rounded-lg p-4">
            <h3 className="font-semibold mb-2">Best For</h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">General tasks, chat</p>
          </div>
          <div className="border border-slate-200 dark:border-slate-800 rounded-lg p-4">
            <h3 className="font-semibold mb-2">Speed</h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">Fast</p>
          </div>
        </div>
        <CodeBlock language="json">
          {`{
  "model": "grok-3",
  "messages": [{"role": "user", "content": "Hello!"}]
}`}
        </CodeBlock>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Grok-3-Thinking</h2>
        <p className="text-slate-600 dark:text-slate-400">
          Enhanced reasoning capabilities for complex problems. Ideal for math, logic puzzles, and multi-step reasoning tasks.
        </p>
        <div className="grid md:grid-cols-3 gap-4">
          <div className="border border-slate-200 dark:border-slate-800 rounded-lg p-4">
            <h3 className="font-semibold mb-2">Context Length</h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">128K tokens</p>
          </div>
          <div className="border border-slate-200 dark:border-slate-800 rounded-lg p-4">
            <h3 className="font-semibold mb-2">Best For</h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">Reasoning, math, logic</p>
          </div>
          <div className="border border-slate-200 dark:border-slate-800 rounded-lg p-4">
            <h3 className="font-semibold mb-2">Speed</h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">Medium</p>
          </div>
        </div>
        <CodeBlock language="json">
          {`{
  "model": "grok-3-thinking",
  "messages": [{"role": "user", "content": "Solve this complex problem..."}]
}`}
        </CodeBlock>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Grok-4-Auto</h2>
        <p className="text-slate-600 dark:text-slate-400">
          Balanced default for production chat workloads when you want Grok-4 quality without switching slugs dynamically.
        </p>
        <div className="grid md:grid-cols-3 gap-4">
          <div className="border border-slate-200 dark:border-slate-800 rounded-lg p-4">
            <h3 className="font-semibold mb-2">Context Length</h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">128K tokens</p>
          </div>
          <div className="border border-slate-200 dark:border-slate-800 rounded-lg p-4">
            <h3 className="font-semibold mb-2">Best For</h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">Production chat, balanced quality</p>
          </div>
          <div className="border border-slate-200 dark:border-slate-800 rounded-lg p-4">
            <h3 className="font-semibold mb-2">Speed</h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">Medium</p>
          </div>
        </div>
        <CodeBlock language="json">
          {`{
  "model": "grok-latest",
  "messages": [{"role": "user", "content": "Summarize this release note."}]
}`}
        </CodeBlock>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Model Comparison</h2>
        <div className="overflow-x-auto">
          <table className="w-full border-collapse">
            <thead>
              <tr className="border-b border-slate-200 dark:border-slate-800">
                <th className="text-left p-3">Model</th>
                <th className="text-left p-3">Context</th>
                <th className="text-left p-3">Best For</th>
                <th className="text-left p-3">Speed</th>
              </tr>
            </thead>
            <tbody>
              <tr className="border-b border-slate-200 dark:border-slate-800">
                <td className="p-3 font-medium">grok-3</td>
                <td className="p-3">128K</td>
                <td className="p-3">General tasks</td>
                <td className="p-3">Fast</td>
              </tr>
              <tr className="border-b border-slate-200 dark:border-slate-800">
                <td className="p-3 font-medium">grok-3-thinking</td>
                <td className="p-3">128K</td>
                <td className="p-3">Reasoning</td>
                <td className="p-3">Medium</td>
              </tr>
              <tr className="border-b border-slate-200 dark:border-slate-800">
                <td className="p-3 font-medium">grok-4-auto</td>
                <td className="p-3">128K</td>
                <td className="p-3">Balanced production chat</td>
                <td className="p-3">Medium</td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>
    </div>
  );
}
