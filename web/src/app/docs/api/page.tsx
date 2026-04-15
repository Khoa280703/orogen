import { ApiEndpointCard } from '@/components/api-endpoint-card';
import { CodeBlock } from '@/components/code-block-with-copy';

export default function ApiReference() {
  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-4xl font-bold mb-4">API Reference</h1>
        <p className="text-lg text-slate-600 dark:text-slate-400">
          Basic API documentation for exact-model chat completions and model discovery.
        </p>
      </div>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Authentication</h2>
        <p className="text-slate-600 dark:text-slate-400">
          All API requests require authentication using a Bearer token. Include your API key in the Authorization header:
        </p>
        <CodeBlock language="bash">
          {'Authorization: Bearer your-api-key-here'}
        </CodeBlock>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Endpoints</h2>

        <div className="space-y-4">
          <ApiEndpointCard
            method="POST"
            path="/v1/chat/completions"
            description="Generate basic text chat completions with optional streaming"
          />
          <ApiEndpointCard
            method="GET"
            path="/v1/models"
            description="List available models"
          />
        </div>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Chat Completions</h2>
        <p className="text-slate-600 dark:text-slate-400">
          This endpoint supports simple text messages and SSE streaming. Tool calling, function calling, and Anthropic-style message APIs are not supported.
        </p>

        <h3 className="text-xl font-semibold">Request</h3>
        <CodeBlock language="json" title="POST /v1/chat/completions">
          {[
            '{',
            '  "model": "grok-3",',
            '  "messages": [',
            '    {',
            '      "role": "system",',
            '      "content": "You are a helpful assistant."',
            '    },',
            '    {',
            '      "role": "user",',
            '      "content": "Hello!"',
            '    }',
            '  ],',
            '  "stream": false',
            '}',
          ]}
        </CodeBlock>

        <h3 className="text-xl font-semibold mt-6">Response</h3>
        <CodeBlock language="json" title="Response">
          {[
            '{',
            '  "id": "chatcmpl-123",',
            '  "object": "chat.completion",',
            '  "created": 1234567890,',
            '  "model": "grok-3",',
            '  "choices": [',
            '    {',
            '      "index": 0,',
            '      "message": {',
            '        "role": "assistant",',
            '        "content": "Hello! How can I help you today?"',
            '      },',
            '      "finish_reason": "stop"',
            '    }',
            '  ],',
            '  "usage": {',
            '    "prompt_tokens": 10,',
            '    "completion_tokens": 10,',
            '    "total_tokens": 20',
            '  }',
            '}',
          ]}
        </CodeBlock>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Streaming Responses</h2>
        <p className="text-slate-600 dark:text-slate-400">
          Set <code>stream: true</code> to receive Server-Sent Events (SSE) with text chunks as they are generated.
        </p>
        <CodeBlock language="python" title="Python streaming example">
          {[
            'import requests',
            '',
            'response = requests.post(',
            '    "https://api.example.com/v1/chat/completions",',
            '    headers={"Authorization": "Bearer your-api-key"},',
            '    json={',
            '        "model": "grok-3",',
            '        "messages": [{"role": "user", "content": "Hello!"}],',
            '        "stream": True',
            '    }',
            ')',
            '',
            'for line in response.iter_lines():',
            '    if line:',
            '        print(line.decode("utf-8"))',
          ]}
        </CodeBlock>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Error Handling</h2>
        <p className="text-slate-600 dark:text-slate-400">
          The API uses standard HTTP status codes and returns JSON error responses.
        </p>
        <CodeBlock language="json" title="Error response">
          {[
            '{',
            '  "error": {',
            '    "message": "Invalid API key",',
            '    "type": "authentication_error",',
            '    "code": "invalid_api_key"',
            '  }',
            '}',
          ]}
        </CodeBlock>

        <div className="mt-4 space-y-2">
          <h3 className="text-lg font-semibold">Error Codes</h3>
          <div className="space-y-2">
            <div className="border border-slate-200 dark:border-slate-800 rounded-lg p-3">
              <code className="text-sm">401 Unauthorized</code>
              <p className="text-sm text-slate-600 dark:text-slate-400">Invalid or missing API key</p>
            </div>
            <div className="border border-slate-200 dark:border-slate-800 rounded-lg p-3">
              <code className="text-sm">429 Too Many Requests</code>
              <p className="text-sm text-slate-600 dark:text-slate-400">Rate limit exceeded</p>
            </div>
            <div className="border border-slate-200 dark:border-slate-800 rounded-lg p-3">
              <code className="text-sm">502 Bad Gateway</code>
              <p className="text-sm text-slate-600 dark:text-slate-400">Upstream Grok account unavailable or external service failed</p>
            </div>
          </div>
        </div>
      </section>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Rate Limits</h2>
        <p className="text-slate-600 dark:text-slate-400">
          Rate limits are applied per API key and vary by subscription plan. Check your dashboard for current limits.
        </p>
      </section>
    </div>
  );
}
