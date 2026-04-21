import { ApiEndpointCard } from '@/components/api-endpoint-card';
import { CodeBlock } from '@/components/code-block-with-copy';

export default function ApiReference() {
  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-4xl font-bold mb-4">API Reference</h1>
        <p className="text-lg text-slate-600 dark:text-slate-400">
          Public API surface for model discovery, chat completions, and Responses-compatible clients.
        </p>
      </div>

      <section className="space-y-4">
        <h2 className="text-2xl font-semibold">Authentication</h2>
        <p className="text-slate-600 dark:text-slate-400">
          <code>GET /v1/models</code> can be called anonymously for the public catalog, or with your API key for a plan-filtered view. Generation endpoints require a Bearer token once at least one API key exists in the system; before that, local bootstrap requests are accepted without auth.
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
            description="List plan-visible public models"
          />
          <ApiEndpointCard
            method="POST"
            path="/v1/responses"
            description="Responses-compatible endpoint for text-first clients and Codex CLI setup"
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
            '  "model": "your-model-id",',
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
            '  "model": "your-model-id",',
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
        <h2 className="text-2xl font-semibold">Responses API</h2>
        <p className="text-slate-600 dark:text-slate-400">
          Use <code>/v1/responses</code> for text-first clients that expect the OpenAI Responses wire API. The route accepts optional reasoning controls through <code>reasoning_effort</code> or <code>reasoning.effort</code>. Tool declarations in the top-level <code>tools</code> array are accepted for client compatibility and ignored on the current gateway path, while actual tool/function/computer payloads and image inputs are still rejected.
        </p>
        <CodeBlock language="json" title="POST /v1/responses">
          {[
            '{',
            '  "model": "your-model-id",',
            '  "input": "Draft a concise release note for this deployment.",',
            '  "reasoning": { "effort": "medium" },',
            '  "stream": false',
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
            '    "https://your-duanai-domain.com/v1/chat/completions",',
            '    headers={"Authorization": "Bearer your-api-key"},',
            '    json={',
            '        "model": "your-model-id",',
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
        <p className="text-sm text-slate-600 dark:text-slate-400">
          Current behavior: streaming routes return <code>text/event-stream</code>. If routing fails before any upstream stream starts, <code>/v1/chat/completions</code> emits an SSE error payload and <code>/v1/responses</code> emits <code>response.failed</code> followed by <code>[DONE]</code>.
        </p>
        <p className="text-sm text-slate-600 dark:text-slate-400">
          If the upstream stream has already emitted content and then fails, the router fails closed instead of switching accounts mid-stream. <code>/v1/chat/completions</code> closes with a final SSE error payload and no trailing <code>[DONE]</code>, while <code>/v1/responses</code> emits <code>response.failed</code> followed by <code>[DONE]</code>. Partial chunks that were already delivered are not replayed.
        </p>
        <p className="text-sm text-slate-600 dark:text-slate-400">
          When the routed provider emits reasoning/thinking tokens on <code>/v1/responses</code>, the stream also includes <code>response.reasoning_summary_*</code> events before the final completion envelope.
        </p>
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
            '  }',
            '}',
          ]}
        </CodeBlock>
        <p className="text-sm text-slate-600 dark:text-slate-400">
          Debug builds may include an additional <code>error.debug</code> field for local troubleshooting, but production responses only guarantee <code>error.message</code>.
        </p>

        <div className="mt-4 space-y-2">
          <h3 className="text-lg font-semibold">Error Codes</h3>
          <div className="space-y-2">
            <div className="border border-slate-200 dark:border-slate-800 rounded-lg p-3">
              <code className="text-sm">400 Bad Request</code>
              <p className="text-sm text-slate-600 dark:text-slate-400">Unsupported function/computer payloads, tool message roles, or image inputs on routes that are still text-first</p>
            </div>
            <div className="border border-slate-200 dark:border-slate-800 rounded-lg p-3">
              <code className="text-sm">403 Forbidden</code>
              <p className="text-sm text-slate-600 dark:text-slate-400">The requested public model is not available in the current plan</p>
            </div>
            <div className="border border-slate-200 dark:border-slate-800 rounded-lg p-3">
              <code className="text-sm">503 Service Unavailable</code>
              <p className="text-sm text-slate-600 dark:text-slate-400">No healthy upstream accounts are currently selectable for the requested provider route</p>
            </div>
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
              <p className="text-sm text-slate-600 dark:text-slate-400">Upstream routed account unavailable or external service failed</p>
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
