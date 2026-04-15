---
type: brainstorm
date: 2026-04-09
slug: media-studio-pivot
status: agreed
---

# Brainstorm: Pivot to Media Studio Platform

## Problem
- Current: OpenAI-compatible API proxy for Grok (web scraping, account pool)
- Grok web interface only returns conversational text — no tool calling, no structured output
- Cannot integrate into Claude Code, agent CLIs, or any tool-calling workflow
- xAI official API supports this but account pool subscriptions don't include xAI API access
- Cost arbitrage via account pool is the business advantage — must keep

## Decision
Pivot from developer API proxy → **Consumer Media Studio + Developer API**

## Product Direction
- **Chat**: conversational UI with streaming, conversation history
- **Image Gen**: prompt editor, generation gallery, download, history
- **Video Gen**: phase 2 (later)
- **Target**: consumer (web UI) + developer (API access)
- **Business**: free tier (few uses/day) + buy credits + subscription tiers

## Architecture
- **Provider abstraction**: each provider (Grok, future Flux/SD/DALL-E) has isolated flow
- **Keep**: account pool, proxy system, billing/plans, user auth, migrations
- **Add**: conversation storage (DB), media storage, generation history
- **Reduce**: OpenAI-compatible format layer
- **Frontend**: rewrite user-facing (media studio UX), refactor admin panel

## MVP Scope
1. Chat — conversation UI, streaming, history persistence
2. Image Gen — prompt → generate → gallery → download
3. Billing — free tier + credits + subscriptions (backend exists)

## Risks
- **ToS/fragility**: accepted, mitigated by clean provider isolation
- **Competition**: differentiate on pricing (cheaper via account pool)
- **Scaling**: need more accounts as user base grows

## Next Steps
- Create detailed implementation plan with phases
