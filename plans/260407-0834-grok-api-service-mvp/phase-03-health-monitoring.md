# Phase 3: Health Monitoring

## Overview
- Priority: Medium
- Status: complete
- Track account health, auto-pause bad accounts, structured logging

## Requirements
- Track success/fail per account
- Auto-pause account after N consecutive failures
- Log each request: timestamp, api_key, account, model, status, latency
- GET /admin/accounts endpoint to view account status

## Files to Modify
- `src/account/types.rs` — add `fail_count: u32`, `success_count: u64` to AccountEntry
- `src/account/pool.rs` — add `mark_success()`, `mark_failure()` methods, auto-pause logic
- `src/api/chat_completions.rs` — call mark_success/mark_failure after each request
- `src/api/mod.rs` — add GET /admin/accounts route
- `src/main.rs` — structured request logging

## Implementation Steps
1. Add counters to AccountEntry (not persisted, runtime only)
2. mark_failure(): increment fail_count, pause if >= 3 consecutive
3. mark_success(): reset fail_count, increment success_count
4. Update send_with_retry() to call appropriate markers
5. Add /admin/accounts returning account name, active, counts, last_used, proxy
6. Add tracing::info! per request with key details

## Success Criteria
- Accounts auto-pause after 3 consecutive failures
- /admin/accounts shows real-time status
- Each request logged with key metadata
