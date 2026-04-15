# Deep Research: Cloudflare Bypass on grok.com with wreq

**Date:** 2026-04-07
**Status:** BRUTALLY HONEST ASSESSMENT

---

## TL;DR

**wreq TLS fingerprinting alone is NOT sufficient to bypass Cloudflare on grok.com.** The 403 is almost certainly caused by **IP reputation**, not TLS fingerprint mismatch. grok2api-rs has the SAME problem — its issue #11 reports identical 403s on datacenter IPs. The projects that "work" either: (1) run on clean residential/non-flagged IPs, or (2) use cf_clearance obtained via headless browser (FlareSolverr).

---

## 1. What grok2api-rs ACTUALLY Does (Source Code Analysis)

Read the actual source: `src/services/grok/wreq_client.rs`, `src/services/grok/chat.rs`

### wreq Client Configuration
```rust
// grok2api-rs build_client (wreq_client.rs)
Client::builder()
    .emulation(Emulation::Chrome136)  // same as ours
    .timeout(Duration::from_secs(timeout))
    .connect_timeout(Duration::from_secs(timeout.clamp(5, 30)))
    .build()
```

### Headers (chat.rs lines 150-195)
**IDENTICAL to our headers.** Same order, same values:
- Accept, Accept-Encoding, Accept-Language, Baggage, Cache-Control, Content-Type
- Origin, Pragma, Priority, Referer
- Sec-Ch-Ua (Chrome 136), Sec-Ch-Ua-Arch ("arm"), Sec-Ch-Ua-Mobile ("?0")
- Sec-Ch-Ua-Platform ("macOS"), Sec-Fetch-* headers
- User-Agent (Chrome 136 Mac), x-statsig-id, x-xai-request-id
- Cookie: `sso={token};cf_clearance={cf}` (if cf_clearance configured)

### Dependencies (Cargo.toml)
```toml
wreq = { version = "6.0.0-rc.27", features = ["stream", "json", "gzip", "brotli", "deflate", "zstd"] }
wreq-util = "3.0.0-rc.9"
```
**Same versions and features as our project.** No `cookies` feature enabled (manual cookie header injection).

### Key Finding
Our code is essentially a copy of grok2api-rs's wreq setup. **There is no secret sauce in grok2api-rs that we're missing.** The code is practically identical.

---

## 2. grok2api-rs Issues — Is It Actually Working?

### Issue #11 (Open, April 2026): "403, cannot use"
> "Deployed on own server, DOCKER mode, imported KEY returns 403. But the **same KEY on CF version works normally**. Server is in USA, US IP."

This is EXACTLY our problem. User reports Docker deployment gets 403, while Cloudflare Workers version works with same credentials.

### Issue #1 (Open): "Add Cloudflare deployment method"
> "Can you add Cloudflare deployment support like TQZHR/grok2api?"

People want CF Workers deployment because direct server deployment gets 403.

### No Closed 403 Issues
There are NO resolved 403 issues in grok2api-rs. The project has 11 open issues, several about 403.

### Verdict: grok2api-rs has the SAME 403 problem as us on datacenter IPs.

---

## 3. chenyme/grok2api (Python) — How They Handle CF

### Multi-Layer Approach (from DeepWiki analysis):
1. **FlareSolverr** — headless Chrome service that solves CF challenges automatically
2. **curl_cffi** — Python HTTP client with browser TLS fingerprint (like wreq for Python)
3. **cf_clearance cookie** — obtained from FlareSolverr, injected into all requests
4. **Automatic refresh** — background scheduler refreshes cf_clearance every 3600s
5. **IP+UA+curl_cffi+cf_clearance** — ALL FOUR must match (per issue #77 comments)

### Issue #77 (Critical Finding):
> "IP, UA, cf_clearance, and curl_cffi browser fingerprint — all 4 must be consistent to avoid 403"

This means even with correct TLS fingerprint, you STILL need cf_clearance from a browser challenge solution.

### Issue #415 (March 2026): Alibaba Cloud US servers get 403
Multiple users report Chinese cloud providers (Alibaba, Tencent) in US regions get 403 even with cf_clearance configured. Some users report the cf_clearance workaround is "ineffective."

### Issue #173: Request for automatic cf_clearance
People need automated cf_clearance retrieval because manual entry is too fragile.

---

## 4. wreq-Specific Findings

### wreq Issue #911 (Closed): WebSocket 403 from Cloudflare
Key comment by `echelon`:
> "Cloudflare detects HTTP/1.1 headers that are lower cased and flag all requests as bot activity. Since Websocket requests begin their life as HTTP 1.1 requests, they're impacted by this."
> "I've ruled out the SSL handshake and fingerprint and reproduced this by simply changing working requests to have lower case HTTP header names."

**This is NOT our issue** — we're using HTTP/2 POST requests, not WebSocket. And wreq's emulation handles header casing correctly for HTTP/2.

### What wreq Emulation Provides:
- TLS fingerprint matching Chrome 136 (JA3/JA4)
- HTTP/2 settings: `initial_window_size=6291456`, `initial_connection_window_size=15728640`
- HTTP/2 pseudo-header order: `:method, :authority, :scheme, :path`
- Client hints headers auto-generated
- Brotli/zstd compression support

### What wreq Does NOT Provide:
- Cookie store is NOT enabled (feature `cookies` not in our Cargo.toml) — but we do manual Cookie header injection, same as grok2api-rs
- No JavaScript execution capability
- No CF challenge solving
- No Turnstile CAPTCHA solving

### wreq `cookies` Feature
Available but NOT needed. Both our code and grok2api-rs use manual `Cookie` header injection. The `cookies` feature is for automatic cookie jar management across redirects — not relevant here since we hit a single endpoint.

---

## 5. Root Cause Analysis: Why 403?

Cloudflare's detection on grok.com is **multi-layered**:

### Layer 1: IP Reputation (PRIMARY CAUSE)
- Datacenter IPs are flagged by Cloudflare's threat intelligence
- Chinese cloud providers in US (Alibaba, Tencent) are heavily flagged
- CF Worker shared IPs also get flagged (per grok-playground docs)
- **Clean residential/non-flagged datacenter IPs may work WITHOUT cf_clearance**

### Layer 2: TLS Fingerprint
- wreq with Chrome136 emulation handles this correctly
- JA3/JA4 should match real Chrome
- **This is NOT our problem** — our TLS fingerprint is correct

### Layer 3: HTTP/2 Fingerprint
- wreq handles pseudo-header ordering, window sizes, etc.
- **This is NOT our problem**

### Layer 4: Behavioral Analysis
- cf_clearance cookie proves a browser previously solved a challenge from this IP
- Without it, flagged IPs get immediate 403
- With it, requests from same IP+UA pass through

### Layer 5: JavaScript Challenge (Managed Challenge)
- Some IPs trigger Cloudflare's managed challenge (JS execution required)
- Cannot be solved by HTTP clients alone
- Requires headless browser (FlareSolverr, nodriver, etc.)

---

## 6. Is cf_clearance Mandatory?

**It depends on IP reputation:**

| IP Type | cf_clearance Needed? | Notes |
|---------|---------------------|-------|
| Clean residential | Probably NO | May pass on TLS fingerprint alone |
| Clean non-flagged datacenter | Maybe NO | Hit or miss |
| Flagged datacenter (Chinese cloud US) | YES | And may still fail |
| CF Worker shared IPs | YES | Often blocked entirely |
| VPN/proxy IPs | YES | High risk of blocking |

**For our use case (server deployment), cf_clearance is almost certainly required.**

---

## 7. Practical Solutions (Ranked by Reliability)

### Option A: cf_clearance via FlareSolverr (RECOMMENDED)
**What:** Run FlareSolverr as a sidecar Docker container. It launches headless Chrome, navigates to grok.com, solves CF challenge, returns cf_clearance cookie.

**Implementation:**
1. Add FlareSolverr container to docker-compose
2. On startup (and every ~30-60 min), call FlareSolverr API
3. Extract cf_clearance + User-Agent from response
4. Inject both into all wreq requests
5. **IP must match** — FlareSolverr and wreq must use same outbound IP

**Rust integration:**
```rust
// Call FlareSolverr API
let resp = reqwest::Client::new()
    .post("http://flaresolverr:8191/v1")
    .json(&json!({
        "cmd": "request.get",
        "url": "https://grok.com",
        "maxTimeout": 60000
    }))
    .send().await?;
// Extract cf_clearance from response cookies
// Extract User-Agent from response
```

**Pros:** Battle-tested, used by chenyme/grok2api, auto-refresh capable
**Cons:** Adds Docker dependency (~500MB image), Chrome resource overhead

### Option B: Clean IP + No cf_clearance
**What:** Deploy on a provider with clean IPs (render.com, fly.io, Hetzner, non-Chinese US providers).

**Evidence:** grok2api issue #77 comment mentions render.com works without 403 while Alibaba Cloud doesn't.

**Pros:** Simplest solution, no extra dependencies
**Cons:** IP may get flagged over time, no guarantee

### Option C: Residential Proxy
**What:** Route wreq requests through a residential proxy service.

**Implementation:** wreq supports proxy via builder:
```rust
Client::builder()
    .emulation(Emulation::Chrome136)
    .proxy(wreq::Proxy::all("http://user:pass@proxy:port")?)
    .build()?
```

**Pros:** Very clean IPs, low detection risk
**Cons:** Cost ($$$), latency, dependency on proxy provider

### Option D: Cloudflare Workers Deployment (UNRELIABLE)
**What:** Deploy the proxy as a CF Worker so requests to grok.com come from CF's own network.

**Evidence:** grok2api-rs issue #11 user says "CF version works, Docker doesn't." BUT grok-playground docs warn: "Worker may be blocked if deployed from an IP that Grok flags as suspicious." CF Worker IPs are shared and often flagged.

**Pros:** Sometimes bypasses CF challenge entirely (CF-to-CF traffic)
**Cons:** Unreliable, grok.com may block CF Worker IPs, can't run Rust natively

### Option E: Nodriver/Camoufox for cf_clearance (Most Robust)
**What:** Use Python nodriver (async, undetected Chrome) or Camoufox (Firefox anti-detect) to get cf_clearance. More reliable than FlareSolverr in 2026.

**Pros:** Most detection-resistant, actively maintained
**Cons:** Python dependency, more complex integration

---

## 8. TLS Fingerprint Verification Tools

To verify our wreq TLS fingerprint matches Chrome:

| Tool | URL | What it checks |
|------|-----|----------------|
| TLS Fingerprint Analyzer | https://tlsinfo.me/ | JA3, JA4 hashes |
| anti-detect.com | https://anti-detect.com/tools/tls-fingerprint | JA3/JA4 + TLS details |
| Scrapfly | https://scrapfly.io/web-scraping-tools/ja3-fingerprint | JA3/JA4 vs known browsers |
| TrustMyIP | https://trustmyip.com/ja3-fingerprint | JA3 hash comparison |

**Test method:** Make a request from our wreq client to these endpoints, compare JA3/JA4 hash with real Chrome 136. If they match, TLS fingerprint is NOT the issue (confirming IP reputation as root cause).

---

## 9. IP Reputation Check

To verify if our server IP is flagged:

1. **Browser test:** From server, use `curl -I https://grok.com` — if you get a 403 or redirect to CF challenge page, IP is flagged
2. **ipinfo.io:** Check `curl https://ipinfo.io/json` — look at hosting/datacenter classification
3. **CF challenge test:** Open `https://grok.com` in a headless browser from the server — if CF challenge appears, IP is flagged
4. **AbuseIPDB:** Check `https://www.abuseipdb.com/check/{IP}` for reputation score

---

## 10. Comparison: Our Code vs grok2api-rs

| Aspect | Our Code | grok2api-rs | Match? |
|--------|----------|-------------|--------|
| wreq version | 6.0.0-rc.27 | 6.0.0-rc.27 | YES |
| wreq-util | 3.0.0-rc.9 | 3.0.0-rc.9 | YES |
| Emulation | Chrome136 | Chrome136 (configurable) | YES |
| Headers | Full Chrome 136 set | Full Chrome 136 set | YES |
| Cookie injection | Manual header | Manual header | YES |
| cf_clearance | Optional field | Optional config | YES |
| statsig-id | Random base64 | Random base64 | YES |
| FlareSolverr | NO | NO | YES |
| Cookie store feature | NO | NO | YES |

**Our code is functionally identical to grok2api-rs.** The 403 is NOT a code problem.

---

## 11. Recommended Action Plan

### Immediate (verify root cause):
1. Test TLS fingerprint against tlsinfo.me from our wreq client
2. Check server IP reputation (ipinfo.io, browser test from server)
3. Try a request from a different, clean IP (residential VPN on laptop) with same code

### Short-term fix:
4. Integrate FlareSolverr as sidecar container
5. Add background cf_clearance refresh (every 30-60 min)
6. Ensure IP+UA+cf_clearance consistency across requests

### Medium-term:
7. Consider switching to a hosting provider with cleaner IPs
8. Implement proxy rotation for when cf_clearance fails
9. Add 403 retry logic that triggers cf_clearance refresh

---

## Unresolved Questions

1. **Which specific hosting provider has clean-enough IPs for grok.com?** render.com reported working by one user, but no systematic testing data.
2. **Does grok.com use Turnstile or managed challenge?** If Turnstile, FlareSolverr may struggle (some Turnstile variants are harder).
3. **How long does cf_clearance last?** chenyme/grok2api uses 3600s refresh interval, but actual expiry may vary.
4. **Does wreq Chrome136 JA3 hash EXACTLY match real Chrome 136?** Needs verification via fingerprint testing tools.
5. **Will grok.com tighten detection further?** This is an arms race — any solution may break at any time.
