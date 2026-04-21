import { CodeBlock } from '@/components/code-block-with-copy';

export default function PythonGuide() {
  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-4xl font-bold mb-4">Python Integration</h1>
        <p className="text-lg text-slate-600 dark:text-slate-400">
          Use the gateway with Python through the OpenAI SDK.
        </p>
      </div>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Installation</h2>
        <CodeBlock language="bash">
          {'pip install openai'}
        </CodeBlock>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Basic Usage</h2>
        <CodeBlock language="python" title="chat.py">
          {[
            'from openai import OpenAI',
            '',
            '# Initialize the client',
            'client = OpenAI(',
            '    api_key="your-api-key",',
            '    base_url="https://your-duanai-domain.com/v1"',
            ')',
            '',
            '# Make a request',
            'response = client.chat.completions.create(',
            '    model="your-model-id",',
            '    messages=[',
            '        {"role": "system", "content": "You are a helpful assistant."},',
            '        {"role": "user", "content": "Hello!"}',
            '    ]',
            ')',
            '',
            'print(response.choices[0].message.content)',
          ]}
        </CodeBlock>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Streaming Responses</h2>
        <CodeBlock language="python" title="stream.py">
          {[
            'from openai import OpenAI',
            '',
            'client = OpenAI(',
            '    api_key="your-api-key",',
            '    base_url="https://your-duanai-domain.com/v1"',
            ')',
            '',
            '# Stream the response',
            'stream = client.chat.completions.create(',
            '    model="your-model-id",',
            '    messages=[{"role": "user", "content": "Tell me a story."}],',
            '    stream=True',
            ')',
            '',
            'for chunk in stream:',
            '    if chunk.choices[0].delta.content:',
            '        print(chunk.choices[0].delta.content, end="", flush=True)',
            'print()',
          ]}
        </CodeBlock>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Advanced Parameters</h2>
        <CodeBlock language="python" title="advanced.py">
          {[
            'response = client.chat.completions.create(',
            '    model="your-model-id",',
            '    messages=[{"role": "user", "content": "Write a poem."}],',
            '    temperature=0.8,        # Creativity (0-2)',
            '    max_tokens=500,         # Max response length',
            '    top_p=0.9,             # Nucleus sampling',
            '    frequency_penalty=0.1, # Reduce repetition',
            '    presence_penalty=0.1   # Encourage new topics',
            ')',
          ]}
        </CodeBlock>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Error Handling</h2>
        <CodeBlock language="python" title="error-handling.py">
          {[
            'from openai import OpenAI, APIError, RateLimitError',
            '',
            'client = OpenAI(',
            '    api_key="your-api-key",',
            '    base_url="https://your-duanai-domain.com/v1"',
            ')',
            '',
            'try:',
            '    response = client.chat.completions.create(',
            '        model="your-model-id",',
            '        messages=[{"role": "user", "content": "Hello!"}]',
            '    )',
            'except RateLimitError:',
            '    print("Rate limit exceeded, please wait")',
            'except APIError as e:',
            '    print(f"API error: {e}")',
          ]}
        </CodeBlock>
        <p className="text-sm text-slate-600 dark:text-slate-400">
          Use <code>GET /v1/models</code> to choose a plan-visible model slug instead of hard-coding upstream names.
        </p>
      </section>
    </div>
  );
}
