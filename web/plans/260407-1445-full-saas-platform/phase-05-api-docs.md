# Phase 5: API Docs + Guides

## Overview
- Priority: Medium
- Status: complete
- Completed: 2026-04-07

## Pages

### Docs Home (`/docs`)
- Quick start guide
- Navigation sidebar

### API Reference (`/docs/api`)
- Endpoints: /v1/chat/completions, /v1/models
- Request/response examples (curl, Python, Node.js)
- Authentication: Bearer token
- Streaming vs non-streaming
- Error codes + handling
- Rate limits explanation

### Setup Guides (`/docs/guides/*`)
- `/docs/guides/quickstart` — 5-minute setup
- `/docs/guides/python` — Python SDK (openai library)
- `/docs/guides/nodejs` — Node.js SDK
- `/docs/guides/curl` — curl examples
- `/docs/guides/langchain` — LangChain integration
- `/docs/guides/chatbox` — Chatbox app setup

### Models (`/docs/models`)
- Available models: grok-3, grok-3-thinking, grok-latest
- Model capabilities, context length
- Pricing per model

### FAQ (`/docs/faq`)
- Common questions + answers

## Approach: MDX with @next/mdx
- Write docs as .mdx files (Markdown + JSX)
- Code blocks with syntax highlighting (rehype-pretty-code)
- Copy button on code blocks
- Sidebar auto-generated from file structure
- i18n: separate mdx files per locale (`docs/en/`, `docs/vi/`)

## File Structure
```
src/app/(public)/docs/
├── layout.tsx          — docs layout with sidebar
├── page.tsx            — docs home
├── api/page.mdx        — API reference
├── models/page.mdx     — model list
├── faq/page.mdx
└── guides/
    ├── quickstart/page.mdx
    ├── python/page.mdx
    ├── nodejs/page.mdx
    ├── curl/page.mdx
    └── langchain/page.mdx
```

## Components
- `src/components/docs-sidebar.tsx` — navigation tree
- `src/components/code-block.tsx` — syntax highlight + copy
- `src/components/api-endpoint.tsx` — styled endpoint card (method, path, description)

## Dependencies
- `@next/mdx`, `@mdx-js/react`
- `rehype-pretty-code`, `shiki` (syntax highlighting)

## Implementation Steps
1. Setup @next/mdx in next.config.ts
2. Create docs layout with sidebar
3. Write API reference (most important)
4. Write quickstart guide
5. Write language-specific guides (Python, Node.js, curl)
6. Add code block component with copy
7. Add i18n support for docs

## Success Criteria
- Docs pages render MDX correctly
- Code blocks have syntax highlighting + copy
- Sidebar navigation works
- User can follow quickstart and make first API call
