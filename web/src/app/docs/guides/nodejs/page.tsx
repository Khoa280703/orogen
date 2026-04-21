import { CodeBlock } from '@/components/code-block-with-copy';

export default function NodejsGuide() {
  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-4xl font-bold mb-4">Node.js Integration</h1>
        <p className="text-lg text-slate-600 dark:text-slate-400">
          Build Node.js applications against the gateway with the OpenAI SDK.
        </p>
      </div>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Installation</h2>
        <CodeBlock language="bash">
          {'npm install openai'}
        </CodeBlock>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Basic Usage</h2>
        <CodeBlock language="typescript" title="chat.ts">
          {[
            'import OpenAI from "openai";',
            '',
            'const client = new OpenAI({',
            '  apiKey: "your-api-key",',
            '  baseURL: "https://your-duanai-domain.com/v1"',
            '});',
            '',
            'async function chat() {',
            '  const response = await client.chat.completions.create({',
            '    model: "your-model-id",',
            '    messages: [',
            '      { role: "system", content: "You are a helpful assistant." },',
            '      { role: "user", content: "Hello!" }',
            '    ]',
            '  });',
            '',
            '  console.log(response.choices[0].message.content);',
            '}',
            '',
            'chat();',
          ]}
        </CodeBlock>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Streaming Responses</h2>
        <CodeBlock language="typescript" title="stream.ts">
          {[
            'import OpenAI from "openai";',
            '',
            'const client = new OpenAI({',
            '  apiKey: "your-api-key",',
            '  baseURL: "https://your-duanai-domain.com/v1"',
            '});',
            '',
            'async function streamChat() {',
            '  const stream = await client.chat.completions.create({',
            '    model: "your-model-id",',
            '    messages: [{ role: "user", content: "Tell me a story." }],',
            '    stream: true',
            '  });',
            '',
            '  for await (const chunk of stream) {',
            '    process.stdout.write(chunk.choices[0].delta.content || "");',
            '  }',
            '  console.log();',
            '}',
            '',
            'streamChat();',
          ]}
        </CodeBlock>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Express.js Example</h2>
        <CodeBlock language="typescript" title="server.ts">
          {[
            'import express from "express";',
            'import OpenAI from "openai";',
            '',
            'const app = express();',
            'app.use(express.json());',
            '',
            'const client = new OpenAI({',
            '  apiKey: process.env.DUANAI_API_KEY,',
            '  baseURL: "https://your-duanai-domain.com/v1"',
            '});',
            '',
            'app.post("/api/chat", async (req, res) => {',
            '  const { message } = req.body;',
            '',
            '  const response = await client.chat.completions.create({',
            '    model: "your-model-id",',
            '    messages: [{ role: "user", content: message }]',
            '  });',
            '',
            '  res.json({ response: response.choices[0].message.content });',
            '});',
            '',
            'app.listen(3000, () => console.log("Server running on port 3000"));',
          ]}
        </CodeBlock>
        <p className="text-sm text-slate-600 dark:text-slate-400">
          Keep the model slug aligned with <code>GET /v1/models</code> so your app only shows what the customer plan can actually call.
        </p>
      </section>
    </div>
  );
}
