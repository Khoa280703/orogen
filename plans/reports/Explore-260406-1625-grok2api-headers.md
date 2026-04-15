# Grok2API Request Analysis Report

## Overview
The chenyme/grok2api project is a FastAPI-based reverse proxy that converts Grok's web interface into an OpenAI-compatible API. It sends requests to `https://grok.com/rest/app-chat/conversations/new` with proper headers to mimic browser requests.

## 1. HTTP Headers Sent to Grok API

### Complete Header Set (from `build_headers()`)

**Standard Headers:**
- `Accept-Encoding: gzip, deflate, br, zstd`
- `Accept-Language: zh-CN,zh;q=0.9,en;q=0.8`
- `Baggage: sentry-environment=production,sentry-release=d6add6fb0460641fd482d767a335ef72b9b6abb8,sentry-public_key=b311e0f2690c81f25e2c4cf6d4f7ce1c`
- `Origin: https://grok.com`
- `Priority: u=1, i`
- `Referer: https://grok.com/`
- `Sec-Fetch-Mode: cors`
- `Sec-Fetch-Dest: empty` (or "document" for media)
- `Sec-Fetch-Site: same-origin`
- `User-Agent: [configurable, default Chrome 136]`
- `Content-Type: application/json`
- `Accept: */*`

**Client Hints (Sec-Ch-Ua headers):**
- `Sec-Ch-Ua: "Google Chrome";v="136", "Chromium";v="136", "Not(A:Brand";v="24"`
- `Sec-Ch-Ua-Mobile: ?0` (or ?1 for mobile)
- `Sec-Ch-Ua-Platform: "Windows"` (or macOS, Linux, Android, iOS)
- `Sec-Ch-Ua-Arch: x86` (or arm)
- `Sec-Ch-Ua-Bitness: 64`
- `Sec-Ch-Ua-Model: ""`

**Authentication:**
- `Cookie: sso={token}; sso-rw={token}; [cf_cookies]`

**Critical Anti-Bot Headers:**
- **`x-statsig-id`** - Statsig session identifier (base64-encoded)
- **`x-xai-request-id`** - UUID for request tracking

---

## 2. Header Generation Mechanisms

### A. x-statsig-id Generation

**Location:** `app/services/reverse/utils/statsig.py` - `StatsigGenerator.gen_id()`

**Two Generation Modes:**

**Static Mode (default):**
```
ZTpUeXBlRXJyb3I6IENhbm5vdCByZWFkIHByb3BlcnRpZXMgb2YgdW5kZWZpbmVkIChyZWFkaW5nICdjaGlsZE5vZGVzJyk=
```
(Base64 decodes to: `e:TypeError: Cannot read properties of undefined (reading 'childNodes')`)

**Dynamic Mode (enabled via `app.dynamic_statsig = true`):**
- Randomly generates JavaScript-like error messages simulating browser environment
- Two variants:
  1. `e:TypeError: Cannot read properties of null (reading 'children['{5-char-random}']')`
  2. `e:TypeError: Cannot read properties of undefined (reading '{10-char-random}')`
- Both are base64-encoded before transmission

**Code Reference:**
```python
@staticmethod
def gen_id() -> str:
    dynamic = get_config("app.dynamic_statsig")
    if dynamic:
        if random.choice([True, False]):
            rand = StatsigGenerator._rand(5, alphanumeric=True)
            message = f"e:TypeError: Cannot read properties of null (reading 'children['{rand}']')"
        else:
            rand = StatsigGenerator._rand(10)
            message = f"e:TypeError: Cannot read properties of undefined (reading '{rand}')"
        return base64.b64encode(message.encode()).decode()
    return "ZTpUeXBlRXJyb3I6IENhbm5vdCByZWFkIHByb3BlcnRpZXMgb2YgdW5kZWZpbmVkIChyZWFkaW5nICdjaGlsZE5vZGVzJyk="
```

### B. x-xai-request-id Generation

**Location:** `app/services/reverse/utils/headers.py` - `build_headers()`

**Generation Method:**
```python
headers["x-xai-request-id"] = str(uuid.uuid4())
```
Simply generates a new UUID for each request (e.g., `"a1b2c3d4-e5f6-47g8-h9i0-j1k2l3m4n5o6"`)

### C. Cookie (SSO Token) Building

**Location:** `app/services/reverse/utils/headers.py` - `build_sso_cookie()`

**Format:**
```
sso={token}; sso-rw={token}; [cf_cookies]; cf_clearance={clearance_token}
```

**Cloudflare Integration:**
- Automatically includes `cf_clearance` cookie from config if provided
- Supports CF cookie refresh via FlareSolverr integration
- Can auto-refresh CF clearance every 3600 seconds when enabled

### D. Client Hints Generation

**Location:** `app/services/reverse/utils/headers.py` - `_build_client_hints()`

Dynamically generates Sec-Ch-Ua headers based on:
- Configured browser type (default: `chrome136`)
- User-Agent string parsing
- Platform detection (Windows, macOS, Linux, iOS, Android)
- Architecture detection (x86 vs ARM)

---

## 3. Request Endpoint & Payload

**API Endpoint:** `POST https://grok.com/rest/app-chat/conversations/new`

**Request Body (JSON):**
```json
{
  "deviceEnvInfo": {
    "darkModeEnabled": false,
    "devicePixelRatio": 2,
    "screenHeight": 1329,
    "screenWidth": 2056,
    "viewportHeight": 1083,
    "viewportWidth": 2056
  },
  "disableMemory": false,
  "disableSearch": false,
  "disableSelfHarmShortCircuit": false,
  "disableTextFollowUps": false,
  "enableImageGeneration": true,
  "enableImageStreaming": true,
  "enableSideBySide": true,
  "fileAttachments": [],
  "forceConcise": false,
  "forceSideBySide": false,
  "imageAttachments": [],
  "imageGenerationCount": 2,
  "isAsyncChat": false,
  "isReasoning": false,
  "message": "{user_message}",
  "modelMode": "{mode}",  // or "modeId" for premium multi-agent modes
  "modelName": "{model}",
  "responseMetadata": {
    "requestModelDetails": { "modelId": "{model}" }
  },
  "returnImageBytes": false,
  "returnRawGrokInXaiRequest": false,
  "sendFinalMetadata": true,
  "temporary": false,
  "toolOverrides": {},
  "customPersonality": "{optional_instructions}"
}
```

---

## 4. Anti-Bot Error Handling

### Error Response Handling

**Current Implementation in `app_chat.py`:**

1. **Non-200 Status Codes:**
   - Extracts error body from response
   - Logs error details (truncated to 500 chars)
   - Raises `UpstreamException` with status code and body

2. **Retry Logic (`app/services/reverse/utils/retry.py`):**
   - **429 (Rate Limit):** NO RETRY - immediately fails
   - **401 (Auth):** Logs failure via `TokenService.record_fail()`
   - **403 (Forbidden):** May trigger session reset on 403
   - **502 (Bad Gateway):** Standard HTTP error retry
   - Other status codes: Handled through `retry_status_codes` config

3. **Proxy Rotation:**
   - On specific status codes, automatically rotates to next proxy
   - Configurable rotation behavior per status code
   - Exponential backoff: base 0.5s, multiplier 2.0x, max 20s, budget 60s

### Anti-Bot Protection Mechanisms

The project does NOT explicitly handle "Request rejected by anti-bot rules" as a specific error. Instead, it uses:

**1. Cloudflare Challenge Handling:**
   - `cf_clearance` cookie from config
   - FlareSolverr integration for automatic CF challenge solving
   - Auto-refresh capability (3600-second intervals)

**2. Browser Impersonation:**
   - Uses `curl_cffi` with browser fingerprinting (default: `chrome136`)
   - Generates realistic client hints headers
   - Includes device environment info (screen dimensions, pixel ratio)
   - Custom User-Agent strings

**3. Header Spoofing:**
   - Statsig ID generation to simulate JavaScript errors (anti-detection)
   - Proper Sec-Fetch-* headers for CORS compliance
   - Sentry baggage headers for telemetry

**4. Proxy Support:**
   - Supports SOCKS5 (normalized to `socks5h://`) and HTTP proxies
   - Automatic proxy rotation on specific error codes
   - Separate handling for SOCKS vs HTTP (different curl_cffi parameters)

**5. Configuration-Based Resilience:**
   - Retry status codes: 401, 429, 403, 502
   - Session reset on 403 for clean proxy rotation
   - Configurable timeouts for different operations

### Anti-Bot Error Response Flow

If Grok returns an anti-bot rejection:
1. Response status code is checked (typically 403 or 429)
2. Error body is extracted and logged
3. If 429: immediate failure (no retry)
4. If 403: may trigger session reset and proxy rotation
5. If proxy configured: rotate to next proxy and retry
6. After max retries: raise `UpstreamException` with status and body

---

## 5. Configuration Settings

**File:** `config.defaults.toml` or `config.toml`

**Critical Anti-Bot Settings:**

```toml
[proxy]
# Primary proxy to route Grok requests through
base_proxy_url = ""

# Asset proxy for static resources
asset_proxy_url = ""

# Cloudflare clearance token (auto-refreshed if enabled)
cf_clearance = ""

# Complete CF cookies (auto-refreshed by service)
cf_cookies = ""

# FlareSolverr URL for auto-solving CF challenges
flaresolverr_url = ""

# Whether to enable automatic CF refresh
enabled = false

# CF challenge timeout
cf_challenge_timeout = 60

# Browser fingerprint
browser = "chrome136"

# Custom User-Agent
user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36"

# Skip proxy SSL verification
skip_proxy_ssl_verify = false

[app]
# Enable dynamic (randomized) Statsig ID generation
dynamic_statsig = true

# Disable memory feature
disable_memory = false

# Optional custom instructions for chat
custom_instruction = ""

[chat]
# Request timeout in seconds
timeout = 60

[retry]
# Retry on these status codes
retry_status_codes = [401, 429, 403, 502]

# Exponential backoff settings
retry_base_delay = 0.5
retry_backoff_multiplier = 2.0
retry_max_delay = 20
retry_budget = 60
```

---

## 6. Key Implementation Files

| File | Purpose |
|------|---------|
| `app/services/reverse/app_chat.py` | Main request handler, builds headers/payload, executes POST |
| `app/services/reverse/utils/headers.py` | Header building utilities (x-statsig-id, x-xai-request-id, client hints) |
| `app/services/reverse/utils/statsig.py` | StatsigGenerator for x-statsig-id creation |
| `app/services/reverse/utils/retry.py` | Retry logic with status code extraction and proxy rotation |
| `app/services/reverse/utils/session.py` | ResettableSession for connection management with 403 reset |
| `app/core/config.py` | Configuration management with auto-migration |
| `app/core/exceptions.py` | Exception definitions (UpstreamException for API errors) |

---

## 7. Request Flow Summary

```
1. User requests chat completion
2. System retrieves SSO token from token pool
3. build_headers() creates header dict:
   - Sets x-statsig-id = StatsigGenerator.gen_id()
   - Sets x-xai-request-id = uuid.uuid4()
   - Includes Cookie: sso={token}; cf_clearance={clearance}
   - Generates client hints based on browser config
4. build_payload() constructs JSON request body
5. AppChatReverse.request() executes:
   - POST to https://grok.com/rest/app-chat/conversations/new
   - Uses configured proxy (SOCKS or HTTP)
   - Browser impersonation via curl_cffi
   - Timeout settings applied
6. Response handling:
   - 200: Stream response lines
   - 401: Log token failure
   - 403: Trigger proxy rotation if configured
   - 429: Immediate failure (rate limited)
   - Non-200: Wrap in UpstreamException
7. Retry logic applies if status code in retry_status_codes
8. Return streamed response or error
```

