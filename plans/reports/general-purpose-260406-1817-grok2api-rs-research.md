# Research: XeanYu/grok2api-rs - Cloudflare Bypass & Architecture

## 1. Cloudflare Bypass Strategy

### Core approach: TLS Fingerprint Emulation via `wreq`

They do NOT use a browser. They use `wreq` (a Rust HTTP client forked from `reqwest` with TLS fingerprint emulation) to impersonate a real browser at the TLS layer.

**Key crate versions:**
```toml
wreq = { version = "6.0.0-rc.27", features = ["stream", "json", "gzip", "brotli", "deflate", "zstd"] }
wreq-util = "3.0.0-rc.9"
```

**Default emulation:** `Chrome136` (configurable via `grok.wreq_emulation` in config)

**cf_clearance handling:** OPTIONAL. Config has `cf_clearance = ""` by default. If provided, it's appended to the Cookie header alongside the SSO token. But the system works WITHOUT it - the wreq TLS fingerprint alone is sufficient to bypass Cloudflare.

```rust
// From chat.rs - ChatRequestBuilder::build_headers
let cf: String = get_config("grok.cf_clearance", String::new()).await;
let cookie = if cf.is_empty() {
    format!("sso={raw}")
} else {
    format!("sso={raw};cf_clearance={cf}")
};
```

### Why it works without cf_clearance

`wreq` with `Emulation::Chrome136` generates a TLS Client Hello that matches Chrome 136's exact fingerprint (JA3/JA4). Cloudflare's bot detection trusts the TLS fingerprint and doesn't challenge with a JS challenge page, so no cf_clearance cookie is needed.

## 2. wreq Client Configuration

### Client builder (`wreq_client.rs`)

```rust
let mut builder = Client::builder()
    .emulation(emulation)                              // Chrome136 default
    .timeout(Duration::from_secs(timeout_secs.max(1))) // 120s default
    .connect_timeout(Duration::from_secs(timeout_secs.clamp(5, 30)));
// Optional proxy support
```

### Supported emulations (configurable):
- Chrome: 100-143 (default: 136)
- Edge: 101, 122, 127, 131, 134-142
- Firefox: 109, 117, 128, 133, 135-146
- Safari: 15.3, 15.5, 16, 16.5, 17.0, 17.2.1, 17.4.1

### Different emulations per endpoint:
- Chat requests: `grok.wreq_emulation` (default `chrome_136`)
- Usage/rate-limit requests: `grok.wreq_emulation_usage` (separate config, can differ)
- NSFW requests: `grok.wreq_emulation_nsfw` (separate config)

### HTTP Headers (Chrome 136 impersonation)

Exact headers from `ChatRequestBuilder::build_headers`:
```
User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36
Sec-Ch-Ua: "Google Chrome";v="136", "Chromium";v="136", "Not(A:Brand";v="24"
Sec-Ch-Ua-Platform: "macOS"
Sec-Ch-Ua-Arch: arm
Sec-Ch-Ua-Bitness: 64
Sec-Ch-Ua-Mobile: ?0
Sec-Fetch-Dest: empty
Sec-Fetch-Mode: cors
Sec-Fetch-Site: same-origin
Origin: https://grok.com
Referer: https://grok.com/
Accept-Encoding: gzip, deflate, br, zstd
Accept-Language: zh-CN,zh;q=0.9
Cache-Control: no-cache
Pragma: no-cache
Priority: u=1, i
Baggage: sentry-environment=production,sentry-release=d6add6fb0460641fd482d767a335ef72b9b6abb8,...
```

### Statsig ID generation

The `x-statsig-id` header is dynamically generated - base64-encoded fake JS error messages:
```rust
// Randomly picks one of:
"e:TypeError: Cannot read properties of null (reading 'children[\"<random5>\"]')"
"e:TypeError: Cannot read properties of undefined (reading '<random10>')"
```
This mimics Grok's frontend telemetry. Configurable: `grok.dynamic_statsig = true`.

### Other per-request headers:
- `x-xai-request-id`: Random UUID v4 per request

## 3. Account Rotation & Token Management

### Token Pool Architecture

Tokens (SSO cookies) are organized into named pools:
- `ssoBasic` - for standard models (grok-3, grok-4, grok-4-mini, etc.)
- `ssoSuper` - for heavy/premium models (grok-4-heavy)

### Selection algorithm (`pool.rs`)

```rust
pub fn select(&self) -> Option<TokenInfo> {
    // 1. Filter: status == Active AND quota > 0
    // 2. Find max quota among available tokens
    // 3. Keep only tokens with that max quota
    // 4. Random selection among those top-quota tokens
}
```

Strategy: **highest-quota-first with random tie-breaking**. This distributes usage to tokens with the most remaining quota, preventing any single token from being exhausted prematurely.

### Quota management

- Default quota per token: **80 units**
- Cost per request: Low effort = 1, High effort = 4
- When quota hits 0: token status changes to `Cooling`
- Tokens auto-refresh via `UsageService` calling `https://grok.com/rest/rate-limits` to sync real remaining quota

### Token lifecycle states:
- `Active` - available for use
- `Cooling` - quota exhausted, waiting for refresh
- `Expired` - failed too many times (5+ consecutive 401s)
- `Disabled` - manually disabled

### Automatic refresh scheduler

Background task runs every `refresh_interval_hours` (default 8h):
1. Finds all `Cooling` tokens
2. Calls Grok's rate-limits API to check real remaining quota
3. If quota recovered: status -> `Active`
4. If API call fails: status -> `Expired`

### Stale reload

TokenManager reloads from disk every `reload_interval_sec` (default 30s) to pick up external changes to token.json.

### Retry logic

- Retries on status codes: `[401, 429, 403]`
- Max retries: 3 (configurable)
- Backoff: `0.5 * (attempt + 1)` seconds (linear)
- On 401 failure: `fail_count` increments; at 5 failures, token marked `Expired`

## 4. Concurrent Request Architecture

### Per-request client creation

A new `wreq::Client` is built per request (in `chat_via_wreq`). No connection pooling or client reuse across requests. This is intentional - each request gets a fresh TLS session.

### Concurrency limits (config)

```toml
[performance]
assets_max_concurrent = 25
media_max_concurrent = 50
usage_max_concurrent = 25
nsfw_max_concurrent = 10
```

These are semaphore-based limits for different operation types.

### Global token manager

Single `Arc<Mutex<TokenManager>>` singleton. All concurrent requests lock it briefly to:
1. Select a token (fast - just picks from pool)
2. Update quota after use

The Mutex is held only during token selection, not during the HTTP request itself. This means many concurrent requests can be in-flight simultaneously with different tokens.

### Multi-user handling

The project exposes an OpenAI-compatible API (`/v1/chat/completions`). Multiple external users hit this API. The system:
1. Receives request
2. Locks TokenManager briefly to select best available token
3. Releases lock
4. Makes request to Grok with that token
5. Streams response back to user
6. After response, locks TokenManager to update quota/status

There is NO per-user session or per-user token affinity. Any user request can use any available token from the pool.

## 5. Key Takeaways for Our Project

1. **wreq is the key dependency** - `wreq 6.0.0-rc.27` with `Emulation::Chrome136` bypasses CF without browser/cf_clearance
2. **No browser needed** - pure HTTP client with TLS fingerprint emulation
3. **cf_clearance is optional fallback** - only used if manually provided in config
4. **Headers must match the emulated browser** - UA, Sec-Ch-Ua, etc. must be consistent with the Chrome version in the emulation
5. **Fresh client per request** - no connection reuse, fresh TLS handshake each time
6. **Token rotation by highest-quota** - not round-robin, picks token with most remaining quota
7. **Background quota sync** - periodic calls to rate-limits API to recover cooled tokens

## Unresolved Questions

- wreq is still in RC (`6.0.0-rc.27`). Stability and API stability unknown.
- The project uses `reqwest` alongside `wreq` (both in Cargo.toml). `reqwest` is used only for its `HeaderMap` type, not for actual HTTP requests. All HTTP calls go through `wreq`.
- No evidence of rotating User-Agent or headers per request - same static Chrome 136 UA for all requests. May become a fingerprinting risk at scale.
- How well does this hold up under Cloudflare updates? TLS fingerprint databases evolve.
