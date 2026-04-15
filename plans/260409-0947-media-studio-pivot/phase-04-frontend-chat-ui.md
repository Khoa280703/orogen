# Phase 4: Frontend — Chat UI

## Context
- [Phase 3 — Consumer API](./phase-03-consumer-api.md)
- Current user-facing pages: dashboard, keys, usage, billing, settings
- Rewrite: new Chat page as primary user experience

## Overview
- **Priority**: P1
- **Status**: Complete
- **Effort**: 8h

Build conversational chat UI. Sidebar with conversation list, main area with messages + input. SSE streaming for real-time responses. Model selector.

## Key Insights
- Use existing Next.js app router + (app) route group
- Reuse existing user-api.ts pattern for API calls
- SSE streaming via EventSource or fetch + ReadableStream
- Mobile-responsive: sidebar collapses on small screens
- Keep it clean and simple — not a ChatGPT clone, media studio feel

## Architecture

```
(app)/chat/
├── layout.tsx          — chat layout (sidebar + main)
├── page.tsx            — redirect to new or latest conversation
└── [id]/
    └── page.tsx        — conversation view

components/chat/
├── chat-sidebar.tsx        — conversation list + new chat button
├── chat-message-list.tsx   — scrollable message list
├── chat-message.tsx        — single message bubble (user/assistant)
├── chat-input.tsx          — message input + send button + model selector
└── chat-stream-handler.ts  — SSE stream parsing utility
```

### Data Flow

```
User types message → chat-input.tsx
  → POST /api/chat/conversations/:id/messages
  → SSE stream response
  → chat-stream-handler.ts parses events
  → chat-message-list.tsx renders tokens in real-time
  → On done: message saved in conversation state
```

## Related Code Files

| File | Action | Purpose |
|------|--------|---------|
| `web/src/app/(app)/chat/layout.tsx` | CREATE | Chat layout with sidebar |
| `web/src/app/(app)/chat/page.tsx` | CREATE | Chat index (new/latest redirect) |
| `web/src/app/(app)/chat/[id]/page.tsx` | CREATE | Conversation view |
| `web/src/components/chat/chat-sidebar.tsx` | CREATE | Conversation list |
| `web/src/components/chat/chat-message-list.tsx` | CREATE | Message display |
| `web/src/components/chat/chat-message.tsx` | CREATE | Single message |
| `web/src/components/chat/chat-input.tsx` | CREATE | Input + model selector |
| `web/src/components/chat/chat-stream-handler.ts` | CREATE | SSE parsing |
| `web/src/lib/user-api.ts` | MODIFY | Add chat API functions |
| `web/src/components/sidebars/user-sidebar.tsx` | MODIFY | Add Chat nav link |

## Implementation Steps

1. **Update user-api.ts** — add chat functions (~30 lines):
   - `createConversation(model)` → POST /api/chat/conversations
   - `listConversations(limit, offset)` → GET /api/chat/conversations
   - `getConversation(id)` → GET /api/chat/conversations/:id
   - `deleteConversation(id)` → DELETE /api/chat/conversations/:id
   - `sendMessageStream(conversationId, content, model)` → POST, returns ReadableStream

2. **Create chat-stream-handler.ts** (~50 lines):
   - Parse SSE events from fetch Response
   - Yield typed events: `{type: 'token'|'thinking'|'done', content: string}`
   - Handle errors and connection drops

3. **Create chat-message.tsx** (~60 lines):
   - Props: role, content, thinking, timestamp
   - User messages: right-aligned, colored bg
   - Assistant messages: left-aligned, with thinking expandable section
   - Simple markdown rendering (bold, code, lists)

4. **Create chat-message-list.tsx** (~50 lines):
   - Scrollable container, auto-scroll to bottom on new messages
   - Render list of ChatMessage components
   - Loading indicator while streaming
   - Empty state for new conversations

5. **Create chat-input.tsx** (~80 lines):
   - Textarea with auto-resize (shift+enter for newline, enter to send)
   - Send button (disabled while streaming)
   - Model selector dropdown (fetch from /v1/models or /api/models)
   - Character count indicator

6. **Create chat-sidebar.tsx** (~80 lines):
   - "New Chat" button at top
   - List conversations (title, date, model badge)
   - Active conversation highlighted
   - Delete button with confirmation
   - Pagination (load more on scroll)

7. **Create chat layout.tsx** (~30 lines):
   - Two-column: sidebar (280px) + main content
   - Mobile: sidebar hidden, toggle button in header
   - Pass conversation list to sidebar

8. **Create chat/page.tsx** (~20 lines):
   - On mount: create new conversation or redirect to latest
   - Model selection for new conversation

9. **Create chat/[id]/page.tsx** (~120 lines):
   - Fetch conversation + messages on mount
   - Render ChatMessageList + ChatInput
   - Handle sendMessage:
     a. Append user message to state
     b. Call sendMessageStream()
     c. Parse SSE events via chat-stream-handler
     d. Append tokens to assistant message in real-time
     e. On done: finalize message in state
   - Handle errors (show toast/banner)

10. **Update user-sidebar.tsx** — add Chat link (icon: MessageSquare from lucide)

## Todo List
- [x] Update user-api.ts with chat functions
- [x] Create chat-stream-handler.ts
- [x] Create chat-message.tsx
- [x] Create chat-message-list.tsx
- [x] Create chat-input.tsx
- [x] Create chat-sidebar.tsx
- [x] Create chat layout.tsx
- [x] Create chat/page.tsx
- [x] Create chat/[id]/page.tsx
- [x] Update user-sidebar.tsx
- [x] `npm run build` to verify
- [x] Browser smoke test authenticated `/chat` flow against live backend

## Success Criteria
- Can create new conversation, select model
- Messages stream in real-time (token by token)
- Thinking content expandable for reasoning models
- Conversation history persists (page refresh keeps messages)
- Sidebar shows conversation list, can switch between conversations
- Delete conversation works
- Mobile responsive

## Risk Assessment
- **Medium**: SSE streaming in Next.js — need client component (`'use client'`)
- **Low**: shadcn/ui components available for all UI elements
- Auto-scroll behavior can be tricky — simple approach: scroll on each token

## Current Progress
- Authenticated smoke test completed against a standalone web server pointed at the latest backend build
- `/chat` and `/chat/1` load with valid JWT session and chat shell renders correctly
- Consumer chat APIs `list`, `create`, and `detail` were exercised with a real JWT-authenticated user
- Runtime note: message send streaming still depends on external Grok account health, so upstream unauthorized errors remain an operational risk rather than a frontend gap

## Next Steps
- Phase 5: Image Studio (parallel with this phase)
- Phase 6: User Dashboard Rewrite (after this)
