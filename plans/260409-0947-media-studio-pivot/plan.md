---
title: "Media Studio Platform Pivot"
description: "Pivot duanai from OpenAI-compatible API proxy to consumer-facing Media Studio with chat, image gen, and billing"
status: completed
priority: P1
effort: 40h
tags: [feature, frontend, backend, refactor]
created: 2026-04-09
---

# Media Studio Platform Pivot

## Overview

Pivot from developer-facing OpenAI-compatible API proxy → consumer-facing **Media Studio** platform.
Keep backend core (account pool, proxy, billing). Build consumer UI for chat + image gen. Refactor provider layer for multi-provider future.

## Context

- Brainstorm report: `plans/reports/brainstorm-260409-0947-media-studio-pivot.md`
- Current: Rust/Axum backend + Next.js admin/user dashboard
- Problem: Grok web scraping can't support tool calling → useless for agent integration
- Solution: Target consumers with chat/image gen UI + keep API for developers

## Phases

| # | Phase | Status | Effort | Link |
|---|-------|--------|--------|------|
| 1 | DB Migration — conversations + media tables | Complete | 3h | [phase-01](./phase-01-db-conversations-media.md) |
| 2 | Backend Provider Abstraction | Complete | 5h | [phase-02](./phase-02-provider-abstraction.md) |
| 3 | Backend Consumer API — chat + image endpoints | Complete | 6h | [phase-03](./phase-03-consumer-api.md) |
| 4 | Frontend — Chat UI | Complete | 8h | [phase-04](./phase-04-frontend-chat-ui.md) |
| 5 | Frontend — Image Studio UI | Complete | 8h | [phase-05](./phase-05-frontend-image-studio.md) |
| 6 | Frontend — User Dashboard Rewrite | Complete | 5h | [phase-06](./phase-06-user-dashboard-rewrite.md) |
| 7 | Admin Panel Refactor | Complete | 3h | [phase-07](./phase-07-admin-refactor.md) |
| 8 | Cleanup — Remove OpenAI compat layer | Complete | 2h | [phase-08](./phase-08-cleanup-openai-compat.md) |

## Dependencies

- Phase 1 → all other phases (DB schema first)
- Phase 2 → Phase 3 (provider trait before consumer API)
- Phase 3 → Phase 4, 5 (backend endpoints before frontend)
- Phase 4, 5, 6 can run in parallel after Phase 3
- Phase 7 independent (can run anytime)
- Phase 8 last (cleanup after everything works)

## Key Decisions

- **Provider trait**: `ChatProvider` + `ImageProvider` traits, Grok implements both
- **Conversations**: stored in DB (not filesystem), linked to user_id
- **Media storage**: URLs from Grok (no self-hosted storage for MVP)
- **Keep API access**: `/v1/*` routes stay for developer users, but simplified
- **Credits system**: reuse existing billing (plans + balances + transactions)

## Current Notes

- Authenticated smoke tests completed for phases 3-6 using a JWT session created via `/api/auth/google`
- Phase 7 is complete: admin sidebar now links to Conversations + Images, dashboard stats include consumer counts, and admin CRUD/read-only monitoring routes exist at `/admin/conversations` and `/admin/images`
- Phase 8 is complete: `/v1/chat/completions` now runs through `ChatProvider`, rejects tool/function payloads, and `src/api/image_generations.rs` now uses `ImageProvider`
- OpenAI/Anthropic compat cleanup landed: `src/api/anthropic_messages.rs` and `src/api/model_mapping.rs` are gone, and `/v1/messages` routes are no longer registered
- Verification on 2026-04-09: `cargo build` passed in repo root and `npm run build` passed in `web/`, including `/admin/conversations` and `/admin/images`
- Verification on 2026-04-09: post-cleanup `cargo build` passed in repo root; router inspection shows `/v1` now exposes `/models`, `/chat/completions`, `/images/generations`, and `/videos/generations`
- Implementation plan complete. Remaining operational blocker for live generation and full `/v1/chat/completions` smoke is upstream Grok account credentials returning `Unauthorized` or `invalid-credentials`
