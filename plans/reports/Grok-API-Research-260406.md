# Grok API Bypass Techniques - Research Report

## Overview
Analyzed 4 production Grok API wrapper repositories to extract novel Cloudflare bypass and authentication techniques currently not in our implementation.

---

## 1. AIClient-2-API (Node.js with Go TLS Sidecar)

### Language & Stack
- **Primary**: Node.js (TypeScript/JavaScript)
- **Cloudflare Bypass**: Go uTLS sidecar binary (`tls-sidecar`)

### Unique Cloudflare Bypass Technique
**TLS Fingerprint Spoofing via Go uTLS**:
- Uses compiled Go uTLS binary to emulate legitimate browser TLS handshakes
- Custom TLS cipher suite ordering: `TLS_AES_128_GCM_SHA256`, `TLS_AES_256_GCM_SHA384`, `TLS_CHACHA20_POLY1305_SHA256`
- Custom signature algorithms: `ecdsa_secp256r1_sha256`, `rsa_pss_rsae_sha256`, etc.
- TLS configuration details:
  - minVersion: `TLSv1.2`, maxVersion: `TLSv1.3`
  - ALPN Protocols: `['http/1.1']`
  - ECDH curves: `X25519:P-256:P-384`
  - Session timeout: 300 seconds
  - Honor cipher order: false

### Headers
```javascript
x-statsig-id: Dynamic generation with random error messages
- Pattern 1: "e:TypeError: Cannot read properties of null (reading 'children[\"{random}']')"
- Pattern 2: "e:TypeError: Cannot read properties of undefined (reading '{random}')"
- Result: Base64 encoded

x-xai-request-id: UUID4 format
```

### Video Generation Special Headers (Sentry tracing)
```javascript
baggage: "sentry-environment=production,sentry-release=..."
sentry-trace: "{traceId}-{parentId}-0" (parent ID is 16 chars)
traceparent: "00-{traceId}-{parentId}-00"
```

### Unique Techniques
1. **Protocol Conversion Middleware**: Abstraction layer that translates between OpenAI, Claude, and Gemini protocols
2. **Account Pool Management**: Intelligent failover with health checks and automatic degradation strategies
3. **Modular Provider Pattern**: Adding new models requires only 3 integration steps
4. **NSFW Setup via gRPC-Web Protocol**:
   - Uses protobuf encoding for `/auth_mgmt.AuthManagement/UpdateUserFeatureControls`
   - Headers: `content-type: application/grpc-web+proto`, `x-grpc-web: 1`
   - Payload construction with binary protobuf frames
5. **Video Generation Handling**:
   - Creates media posts to get persistent URLs
   - Upscaling endpoint: `/rest/media/video/upscale`
   - Share link creation for public distribution
   - Post ID extraction from response URLs

### Cookie Management
```javascript
Cookie: "sso={token}; sso-rw={token}; cf_clearance={clearance}"
// Token auto-strip "sso=" prefix if present
```

---

## 2. Grok-Api (Python with curl_cffi)

### Language & Stack
- **Python 3.10+**
- **HTTP Client**: `curl_cffi` (browser impersonation: Chrome136)
- **Framework**: FastAPI for REST wrapper

### Cloudflare Bypass
Uses `curl_cffi` with browser impersonation - simpler than uTLS approach but effective.

### Headers (Critical Order Management)
The repository implements **header ordering** to match browser conventions:
```python
class Headers:
    LOAD = {
        "upgrade-insecure-requests": "1",
        "user-agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64)...",
        # ... other headers
    }
    
    C_REQUEST = {
        "next-action": "",  # Next.js specific
        "next-router-state-tree": "%5B%22%22%2C{encoded}",  # Encoded routing state
        "baggage": "",
        "sentry-trace": "",
    }
    
    CONVERSATION = {
        "x-xai-request-id": "",
        "x-statsig-id": "",
        "traceparent": "",  # W3C trace context
    }
    
    @staticmethod
    def fix_order(headers, base) -> dict:
        # Reorder headers to match base order for browser legitimacy
```

### Challenge-Response Authentication Flow
Unique 3-step authorization process:
1. **Step 1 (c_request)**: 
   - Sends multipart form with user public key
   - Receives `anonUserId` from response
   - Extracts binary challenge from hex-encoded response

2. **Step 2 (c_request)**:
   - Sends `anonUserId` and signed challenge
   - Receives SVG animation data and animation number sequence

3. **Step 3 (c_request)**:
   - Final validation with parsed signature

Code snippet from authentication:
```python
# Extract challenge from binary response
start_idx = c_request.content.hex().find("3a6f38362c")  # hex for ":o86,"
challenge_bytes = bytes.fromhex(c_request.content.hex()[start_idx:end_idx])
challenge_dict = Anon.sign_challenge(challenge_bytes, self.keys["privateKey"])
```

### Unique Techniques
1. **Cryptographic Key Generation**: Uses `coincurve` for elliptic curve cryptography
2. **SVG Parsing for CAPTCHA**: Extracts animation data from SVG challenges
3. **Session Persistence**: 
   - Stores: `cookies`, `actions`, `xsid_script`, `baggage`, `sentry_trace`
   - Can resume conversations with `extra_data` parameter
4. **Conversation Continuity**: 
   - Reuses `parentResponseId` for follow-up messages
   - Maintains `anonUserId` and `privateKey` across sessions
5. **Signature Generation**: 
   - Uses `/rest/app-chat/conversations/new` endpoint signature
   - Signature includes verification token, SVG data, and parsed numbers

### Dynamic Headers
```python
x-statsig-id: Parser.generate_sign(endpoint, method, verification_token, svg_data, numbers)
x-xai-request-id: str(uuid4())
sentry-trace: f'{self.sentry_trace}-{uuid4().replace("-", "")[:16]}-0'
traceparent: f"00-{token_hex(16)}-{token_hex(8)}-00"  # W3C format
```

---

## 3. Grok2API-rs (Rust with Axum)

### Language & Stack
- **Rust + Axum web framework**
- **HTTP Client**: Built-in `wreq` library (no curl_cffi dependency)
- **Configurable browser emulation**: Chrome100-143, Edge101-142, Firefox109-145, Safari versions

### Cloudflare Bypass
**Wreq Browser Emulation**:
- Supports 40+ different browser fingerprints via enum-based configuration
- Default: Chrome136
- Configuration: `grok.wreq_emulation` (can be global or per-operation)
- No external dependencies - fully built-in

### Unique Technical Approaches

1. **Configurable Browser Fingerprints**:
```rust
pub enum Emulation {
    Chrome136, Chrome137, Chrome138... Chrome143,
    Edge101... Edge142,
    Firefox109... Firefox146,
    Safari15_3... Safari17_4_1,
}
// Parse from config: "chrome136", "edge135", "firefox143"
```

2. **Token Refresh Architecture**:
```rust
pub struct TokenManager {
    auto_refresh: true,
    refresh_interval_hours: 8,
    failure_threshold: tracked_by_status,
}
```

3. **NSFW Retry Logic with Fallback**:
- Includes stability fixes: failure fallback + error details
- Separate handling for NSFW image generation pathway

4. **Server-Sent Events (SSE) Streaming**:
- Native SSE support in admin interface
- Streaming response handling at HTTP level

5. **Upstream Proxy Configuration**:
```rust
grok.base_proxy_url     // General proxy
grok.asset_proxy_url    // Asset/media proxy (separate)
```

### Dynamic Statsig Generation
```rust
pub async fn gen_id() -> String {
    let dynamic = get_config("grok.dynamic_statsig", true).await;
    if !dynamic {
        return "ZTpUeXBlRXJyb3I6IENhbm5vdCByZWFkIHByb3BlcnRpZXMgb2YgdW5kZWZpbmVkIChyZWFkaW5nICdjaGlsZE5vZGVzJyk=".to_string();
    }
    let msg = if rand::random::<bool>() {
        format!("e:TypeError: Cannot read properties of null (reading 'children[\"{}\"]')", random_str(5, true))
    } else {
        format!("e:TypeError: Cannot read properties of undefined (reading '{}')", random_str(10, false))
    };
    base64::encode(msg)
}
```

---

## 4. Grok2API (Python with FastAPI)

### Language & Stack
- **Python + FastAPI**
- **HTTP Client**: `curl_cffi` with AsyncSession
- **Async/Await architecture**
- **TypeScript for Cloudflare Workers** (secondary implementation)

### Unique Techniques

1. **Dynamic Statsig ID Generation**:
   - Configuration flag: `grok.dynamic_statsig=true`
   - Two patterns randomized:
     - "e:TypeError: Cannot read properties of null (reading 'children['{5-char-alphanum}']')"
     - "e:TypeError: Cannot read properties of undefined (reading '{10-char-alpha}')"
   - Base64 encoded for transmission

2. **Auto-Registration System**:
   - Automated account creation workflow
   - Services: Email registration, birth date setup, user agreement, NSFW configuration
   - **Turnstile CAPTCHA Solver**:
     - Local browser automation (5 threads by default)
     - Integrated solver for account verification

3. **Token Management & Pooling**:
   - Token categorization: "active", "expired", "quota_exhausted"
   - Automatic quota tracking (`quota_known` vs unknown)
   - Distinction between `sso` and `ssoSuper` token types
   - Concurrent pool with auto-failover

4. **Streaming Response Handling**:
   ```python
   async def stream_response():
       try:
           async for line in response.aiter_lines():
               yield line
       finally:
           if session:
               await session.close()
   ```

5. **Message Extraction & Processing**:
   - OpenAI message format conversion
   - Support for: text, image_url, input_audio, file types
   - Video models with unsupported type filtering
   - Attachment upload handling before chat request

6. **Request Header Construction**:
   ```python
   headers = {
       "x-statsig-id": StatsigService.gen_id(),
       "x-xai-request-id": str(uuid.uuid4()),
       "Cookie": f"sso={token};cf_clearance={cf}" if cf else f"sso={token}"
   }
   ```

7. **Payload Configuration**:
   - Device environment info: screen size, viewport, pixel ratio
   - Model mode mapping (FAST, AUTO, EXPERT, HEAVY)
   - Tool overrides for different model variants
   - Image generation count control
   - Return format options (bytes vs. URLs)

8. **Error Recovery**:
   - Status-based retry logic (429, 5xx, network errors)
   - Exponential backoff: `delay * 2^retryCount`
   - Configurable max retries and base delay
   - Usage sync on stream completion

---

## Novel Techniques We Don't Have

### High Priority
1. **TLS Fingerprint Spoofing** (AIClient-2-API)
   - Go uTLS sidecar with custom cipher suites and signature algorithms
   - More robust than curl_cffi for strict Cloudflare detection

2. **Challenge-Response Signature Flow** (Grok-Api)
   - 3-step crypto challenge validation
   - SVG CAPTCHA parsing with animation data
   - Persistent session recovery with cryptographic keys

3. **Wreq Browser Emulation Flexibility** (Grok2API-rs)
   - 40+ browser fingerprint options
   - Switchable per-operation (not global)
   - True Rust/native implementation

4. **Turnstile CAPTCHA Solver** (Grok2API)
   - Local browser automation for account registration
   - Multi-threaded solver process

### Medium Priority
1. **Header Order Normalization** (Grok-Api)
   - Explicit header reordering to match base signature
   - Prevents detection of synthetic header generation

2. **Sentry Tracing Headers** (AIClient-2-API, Grok2API)
   - Full W3C `traceparent` format: `00-{traceId}-{parentId}-00`
   - Sentry-specific `baggage` header for release tracking
   - Complete distributed trace context

3. **gRPC-Web Protocol for NSFW** (AIClient-2-API)
   - Protobuf encoding for account feature updates
   - Separate content-type and headers
   - Binary frame protocol

4. **Video Share Link Generation** (AIClient-2-API)
   - Post creation → Share link generation
   - Persistent media URLs via `imagine-public.x.ai`
   - Video upscaling endpoint support

5. **Device Environment Info** (Grok2API)
   - Screen resolution, viewport, pixel ratio
   - Appears in some request payloads
   - May improve legitimacy

### Low Priority (Incremental)
1. **Separate Upstream Proxies** (Grok2API-rs)
   - `base_proxy_url` for API calls
   - `asset_proxy_url` for media assets
   - More granular proxy routing

2. **Account Pool Degradation** (AIClient-2-API)
   - Health checks + automatic account rotation
   - Error counting and account reliability scoring

3. **Token Status Tracking** (Grok2API)
   - Categorize tokens by status before rotation
   - Quota tracking (known vs. unknown limits)

---

## Implementation Priorities

### Adopt Now
1. Implement full W3C `traceparent` header with 16-char hex trace ID and 8-char span ID
2. Add support for multiple browser emulation profiles (like wreq)
3. Implement header order normalization
4. Add `baggage` header for Sentry tracking
5. Support for `cf_clearance` cookie handling

### Adopt with Testing
1. TLS fingerprint spoofing (requires Go sidecar or Rust integration)
2. Challenge-response signature flow (complex, high maintenance)
3. gRPC-Web protocol support for account features

### Research Further
1. Turnstile CAPTCHA solver integration
2. Video generation workflow (Post creation, share links)
3. Token quota tracking and refresh intervals

---

## Findings Summary

| Aspect | Best Implementation | Source |
|--------|-------------------|--------|
| Cloudflare Bypass | Go uTLS Sidecar | AIClient-2-API |
| Browser Emulation | Wreq (40+ profiles) | Grok2API-rs |
| Auth Flow | Challenge-Response Crypto | Grok-Api |
| Async Streaming | curl_cffi AsyncSession | Grok2API |
| Account Pool | Multi-threaded with Health Checks | AIClient-2-API |
| Trace Headers | Full W3C Traceparent | All |
| Statsig Generation | Dynamic Random Patterns | All |

