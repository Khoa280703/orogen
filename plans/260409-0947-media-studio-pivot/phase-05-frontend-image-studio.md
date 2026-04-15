# Phase 5: Frontend — Image Studio UI

## Context
- [Phase 3 — Consumer API](./phase-03-consumer-api.md)
- Current: no consumer image UI (only API endpoint)
- Goal: Midjourney/Leonardo-style image generation interface

## Overview
- **Priority**: P1
- **Status**: Complete
- **Effort**: 8h

Image studio with prompt input, generation gallery, history. Clean media-studio aesthetic.

## Architecture

```
(app)/images/
├── page.tsx              — image studio (generate + recent)
└── history/
    └── page.tsx          — full generation history

components/images/
├── image-prompt-bar.tsx      — prompt input + generate button
├── image-gallery.tsx         — grid of generated images
├── image-card.tsx            — single image with actions
├── image-generation-status.tsx — loading/progress indicator
└── image-history-list.tsx    — paginated history list
```

### Data Flow

```
User enters prompt → image-prompt-bar.tsx
  → POST /api/images/generate
  → Show loading state (image-generation-status)
  → On response: display images in gallery
  → Save to history automatically
```

## Related Code Files

| File | Action | Purpose |
|------|--------|---------|
| `web/src/app/(app)/images/page.tsx` | CREATE | Image studio page |
| `web/src/app/(app)/images/history/page.tsx` | CREATE | History page |
| `web/src/components/images/image-prompt-bar.tsx` | CREATE | Prompt input |
| `web/src/components/images/image-gallery.tsx` | CREATE | Image grid |
| `web/src/components/images/image-card.tsx` | CREATE | Single image card |
| `web/src/components/images/image-generation-status.tsx` | CREATE | Loading state |
| `web/src/components/images/image-history-list.tsx` | CREATE | History list |
| `web/src/lib/user-api.ts` | MODIFY | Add image API functions |
| `web/src/components/user-sidebar.tsx` | MODIFY | Add Images nav link |

## Implementation Steps

1. **Update user-api.ts** — add image functions (~20 lines):
   - `generateImages(prompt, model?)` → POST /api/images/generate
   - `listImageHistory(limit, offset)` → GET /api/images/history
   - `getImageGeneration(id)` → GET /api/images/history/:id

2. **Create image-card.tsx** (~60 lines):
   - Image thumbnail with hover overlay
   - Actions: download (opens URL), expand (lightbox/modal), copy URL
   - Show image ID badge
   - Aspect ratio container (maintain proportions)

3. **Create image-gallery.tsx** (~40 lines):
   - CSS grid: 2 cols on mobile, 4 cols on desktop
   - Render array of ImageCard
   - Empty state when no images

4. **Create image-generation-status.tsx** (~30 lines):
   - Loading spinner with "Generating..." text
   - Error state with retry button
   - Transition to gallery on completion

5. **Create image-prompt-bar.tsx** (~80 lines):
   - Large textarea for prompt (3-4 lines visible)
   - Generate button (disabled during generation)
   - Model selector (imagine-x-1, imagine-x-1-pro future)
   - Character count
   - Sticky at bottom or top of page

6. **Create image-history-list.tsx** (~70 lines):
   - List of past generations: prompt preview + thumbnail grid + date
   - Click to expand full gallery view
   - Pagination (load more button)
   - Filter by status (completed/failed)

7. **Create images/page.tsx** (~120 lines):
   - Main studio page
   - Top: ImagePromptBar
   - Middle: ImageGenerationStatus (during gen) or ImageGallery (results)
   - Bottom: recent history (last 5 generations)
   - State management:
     a. User submits prompt → set loading
     b. Call generateImages() → await response
     c. On success: show gallery, prepend to recent history
     d. On error: show error state with retry

8. **Create images/history/page.tsx** (~60 lines):
   - Full history page with pagination
   - Render ImageHistoryList
   - Link back to studio

9. **Update user-sidebar.tsx** — add Images link (icon: ImageIcon from lucide)

## Todo List
- [x] Update user-api.ts with image functions
- [x] Create image-card.tsx
- [x] Create image-gallery.tsx
- [x] Create image-generation-status.tsx
- [x] Create image-prompt-bar.tsx
- [x] Create image-history-list.tsx
- [x] Create images/page.tsx
- [x] Create images/history/page.tsx
- [x] Update user-sidebar.tsx
- [x] `npm run build` to verify
- [x] Browser smoke test authenticated `/images` and `/images/history` flow against live backend

## Success Criteria
- Can enter prompt and generate images
- Loading state shows while generating (~10-30s)
- Generated images display in grid
- Can download images (open URL in new tab)
- History persists across sessions
- History page shows past generations with pagination
- Mobile responsive (2-col grid on mobile)

## Risk Assessment
- **Low**: straightforward CRUD + display, no streaming complexity
- **Medium**: image generation takes 10-30s — UX must handle wait gracefully
- Grok image URLs may expire — display warning or cache concern (future)

## Current Progress
- Authenticated smoke test completed for `/images` and `/images/history` using a standalone web server against the latest backend build
- Image history API returns persisted records correctly and the UI renders empty/failed states without crashing
- Runtime note: image generation request currently returns upstream `Unauthorized` from Grok account cookies, but the failure is surfaced and stored in history as expected

## Next Steps
- Phase 6: User Dashboard Rewrite (update landing page for studio)
