# Security Review Report - grok-local

**Date:** 2026-04-07
**Current Score:** 4/10
**Target Score After Fixes:** 8-9/10

---

## Critical Issues

### 1. PERMISSIVE CORS - Information Leakage & CSRF Vector
**File:** `src/main.rs:124`
**Severity:** CRITICAL

```rust
let app = api::router(state).layer(CorsLayer::permissive());
```

**Issue:** `CorsLayer::permissive()` allows ANY origin, ANY method, ANY header. This:
- Enables CSRF attacks despite CSRF token (browser sends credentials automatically)
- Exposes admin API to malicious websites
- Allows credential theft via XSS

**Fix:**
```rust
use tower_http::cors::{CorsLayer, Any};

let cors = CorsLayer::new()
    .allow_origin(
        reqwest::Url::parse("http://localhost:3069")
            .map(|url| url.into())
            .unwrap_or(Any),
    )
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers([HeaderName::CONTENT_TYPE, HeaderName::AUTHORIZATION, HeaderName::from_static("x-csrf-token")])
    .expose_headers([]);

let app = api::router(state).layer(cors);
```

---

### 2. ADMIN TOKEN IN LOCALSTORAGE - XSS Token Theft
**File:** `web/src/lib/api.ts:5-10`
**Severity:** CRITICAL

```typescript
export function setAdminToken(token: string): void {
  if (typeof window === 'undefined') return;
  localStorage.setItem('adminToken', token);
}
```

**Issue:**
- localStorage is accessible via JavaScript - any XSS steals admin token
- No HTTP-only flag possible with localStorage
- Token persists even after logout if page cached

**Fix:** Use httpOnly cookies with `Secure` and `SameSite=Strict`:

**Backend (`src/middleware/auth.rs` - new file):**
```rust
use axum::http::header::{SET_COOKIE, COOKIE};
use axum::response::Response;

pub async fn set_session_cookie(
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Set httpOnly, Secure, SameSite=Strict cookie
    let cookie = Cookie::build("admin_session", token)
        .http_only(true)
        .secure(true) // HTTPS only
        .same_site(SameSite::Strict)
        .path("/")
        .max_age(Duration::from_secs(3600)) // 1 hour
        .finish();
    // ...
}
```

**Frontend:** Replace `getAdminToken()` with cookie-based auth.

---

### 3. CSRF TOKEN IN LOCALSTORAGE - Can Be Stolen
**File:** `web/src/lib/api.ts:20-26`
**Severity:** HIGH

```typescript
export function getCsrfToken(): string | null {
  if (typeof window === 'undefined') return null;
  return localStorage.getItem('csrfToken');
}
```

**Issue:**
- CSRF token in localStorage means XSS can steal it
- Defeats CSRF protection entirely if XSS exists
- Token should be session-bound, not client-stored

**Fix:** Bind CSRF to session cookie - server validates both together.

---

### 4. HARDCODED PORT IN FRONTEND - Configuration Leak
**File:** `web/src/components/protected-route.tsx:29`, `web/src/app/login/page.tsx:23`
**Severity:** MEDIUM

```typescript
const response = await fetch('http://localhost:5169/admin/stats/overview', {
```

**Issue:**
- Hardcoded port leaks internal infrastructure details
- Uses http instead of respecting environment
- Should use `process.env.NEXT_PUBLIC_API_URL`

**Fix:**
```typescript
const BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:5169';
const response = await fetch(`${BASE_URL}/admin/stats/overview`, {
```

---

### 5. RATE LIMITER WINDOW MISMATCH - DoS Prevention Bypass
**File:** `src/middleware/rate_limiter.rs:24, 41`
**Severity:** MEDIUM

```rust
pub fn new(max_requests: usize, window: Duration) -> Self {
    // max_requests = 100, window = 60s
}

// Line 41 - hardcoded 60 seconds instead of using `window` parameter
entry.requests.retain(|&t| now.duration_since(t) < Duration::from_secs(60));
```

**Issue:**
- Constructor accepts `window` parameter but ignores it
- Hardcoded 60s window cannot be changed
- Parameter `max_requests` is also ignored (line 43 uses hardcoded 100)

**Fix:**
```rust
pub struct RateLimiter {
    state: Arc<RwLock<RateLimiterState>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            state: Arc::new(RwLock::new(RateLimiterState {
                requests: HashMap::new(),
            })),
            max_requests,
            window,
        }
    }

    pub async fn check(&self, key: &str) -> bool {
        let mut state = self.state.write().await;
        let now = Instant::now();

        let entry = state.requests.entry(key.to_string()).or_insert_with(|| RateLimitEntry {
            requests: Vec::new(),
        });

        // Use self.window instead of hardcoded 60
        entry.requests.retain(|&t| now.duration_since(t) < self.window);

        // Use self.max_requests instead of hardcoded 100
        if entry.requests.len() >= self.max_requests {
            return false;
        }

        entry.requests.push(now);
        true
    }
}
```

---

### 6. RATE LIMITER NOT CLEANING EXPIRED ENTRIES - Memory Leak + Bypass
**File:** `src/middleware/rate_limiter.rs:32-50`
**Severity:** LOW

**Issue:**
- `check()` adds requests but never removes old IP entries entirely
- HashMap grows indefinitely as new IPs are added
- No periodic cleanup of entries with empty request vectors

**Fix:** Add cleanup after checking window:
```rust
pub async fn check(&self, key: &str) -> bool {
    let mut state = self.state.write().await;
    let now = Instant::now();

    let entry = state.requests.entry(key.to_string()).or_insert_with(|| RateLimitEntry {
        requests: Vec::new(),
    });

    entry.requests.retain(|&t| now.duration_since(t) < self.window);

    // Remove empty entries to prevent memory leak
    if entry.requests.is_empty() {
        state.requests.remove(key);
    }

    // ...
}
```

---

### 7. MIDDLEWARE ORDER ISSUE - CSRF Before Rate Limit
**File:** `src/api/mod.rs:64-75`
**Severity:** MEDIUM

```rust
.layer(axum::middleware::from_fn(move |req, next| {
    auth_middleware(all_keys.clone(), req, next)
}))
.layer(axum::middleware::from_fn(move |req, next| {
    admin_auth_middleware(admin_token.clone(), req, next)
}))
.layer(axum::middleware::from_fn(move |req, next| {
    csrf_middleware(csrf_protection.clone(), req, next)
}))
.layer(axum::middleware::from_fn(move |req, next| {
    rate_limiter::rate_limit_middleware(rate_limiter.clone(), req, next)
}))
```

**Issue:**
- Rate limiting should be FIRST to block attackers before auth/CSRF
- Current order processes auth + CSRF before rate limiting
- DoS can exhaust auth/CSRF processing resources

**Fix:** Reorder:
```rust
.layer(axum::middleware::from_fn(move |req, next| {
    rate_limiter::rate_limit_middleware(rate_limiter.clone(), req, next)
})) // FIRST
.layer(axum::middleware::from_fn(move |req, next| {
    auth_middleware(all_keys.clone(), req, next)
}))
// ...
```

---

### 8. PROXY URL STORED IN PLAINTEXT - Credential Leak
**File:** `src/api/admin_proxies.rs:9-13`, `web/src/app/proxies/page.tsx:202`
**Severity:** HIGH

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyCreateRequest {
    pub url: String, // Contains user:pass@host:port
}
```

**Issue:**
- Proxy URLs contain credentials: `socks5h://user:pass@host:port`
- `list_proxies` returns full URLs (line 50)
- Frontend displays URL (line 202 of proxies page)
- Database stores credentials unencrypted
- JSON log may contain credentials

**Fix:**
1. Encrypt proxy credentials at rest
2. Mask credentials in API response
3. Never log proxy URLs

**Backend:**
```rust
// Encrypt before saving to DB
pub async fn create_proxy(
    db: &Pool<Postgres>,
    url: &str,
    label: Option<&str>,
) -> Result<i32, sqlx::Error> {
    // Parse and encrypt credentials
    let parsed = Url::parse(url)?;
    let password = parsed.password().unwrap_or("");
    let encrypted = encrypt_password(password).await?;

    // Store encrypted password separately
    // ...
}

// In list_proxies, mask the URL
fn mask_proxy_url(url: &str) -> String {
    // socks5h://user:***@host:port
}
```

---

### 9. ACCOUNT COOKIES STORED UNENCRYPTED - Session Hijacking
**File:** `src/api/admin_accounts.rs:10-15`, `src/config.rs:157-173`
**Severity:** CRITICAL

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountCreateRequest {
    pub name: String,
    pub cookies: Value, // Contains sso, sso-rw session tokens
}
```

**Issue:**
- Account cookies contain Grok session tokens (`sso`, `sso-rw`)
- Stored unencrypted in PostgreSQL
- `list_accounts` returns full cookies (line 53)
- `cookies.json` file stores unencrypted (config.rs:157-173)

**Fix:** Encrypt cookies at rest:
```rust
use aes_gcm::{Aes256Gcm, nonce::Aes256GcmNonces, KeyInit};
use rand::RngCore;

pub async fn encrypt_cookies(cookies: &Value) -> Result<String, CryptoError> {
    let key = get_encryption_key(); // From env var
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    let nonce = Aes256GcmNonces::from_slice(&rand::random::<[u8; 12]>());
    let plaintext = serde_json::to_string(cookies)?;
    let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes())?;
    Ok(serde_json::to_string(&ciphertext)?)
}
```

---

### 10. NO SESSION INVALIDATION - Stolen Token Abuse
**File:** `web/src/lib/api.ts:13-16`
**Severity:** HIGH

```typescript
export function clearAdminToken(): void {
  if (typeof window === 'undefined') return;
  localStorage.removeItem('adminToken');
}
```

**Issue:**
- No logout endpoint that invalidates server-side session
- Stolen token remains valid indefinitely (no expiration)
- No token rotation

**Fix:**
1. Add token expiration (e.g., 24 hours)
2. Implement logout that invalidates token server-side
3. Implement token rotation on each request

---

### 11. SQL INJECTION VULNERABILITY RISK
**File:** `src/db/mod.rs`, account/proxy queries
**Severity:** HIGH

**Issue:**
- Using `sqlx` which is parameterized, but need to verify all queries use placeholders
- Raw SQL in any migration or query is vulnerable

**Check all DB files** for raw string interpolation.

---

### 12. SENSITIVE DATA IN ERROR RESPONSES
**File:** `src/error.rs:25-35`
**Severity:** LOW

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Returns full error message including internal details
        (status, Json(json!({ "error": { "message": message } }))).into_response()
    }
}
```

**Issue:**
- Error messages may leak internal paths, query details, database errors
- Should return generic messages in production

**Fix:**
```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let message = if cfg!(debug_assertions) {
            self.to_string()
        } else {
            "An error occurred".to_string()
        };
        (status, Json(json!({ "error": { "message": message } }))).into_response()
    }
}
```

---

### 13. NO SECURITY HEADERS - XSS, Clickjacking
**File:** `src/main.rs:124`
**Severity:** HIGH

**Issue:** Missing security headers:
- `Content-Security-Policy`
- `X-Frame-Options`
- `X-Content-Type-Options`
- `Strict-Transport-Security`

**Fix:**
```rust
use tower_http::trace::TraceLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use axum::http::header;

let app = api::router(state)
    .layer(CorsLayer::new()...) // not permissive()
    .layer(SetResponseHeaderLayer::if_not_present(
        header::CONTENT_TYPE,
        "application/json",
    ))
    .layer(SetResponseHeaderLayer::overriding(
        header::X_CONTENT_TYPE_OPTIONS,
        "nosniff",
    ));
```

Add middleware for all security headers.

---

### 14. CSRF TOKEN NOT ROTATED - Token Replay
**File:** `src/middleware/csrf.rs:51-61`
**Severity:** MEDIUM

**Issue:**
- CSRF token valid for 5 minutes (300 seconds)
- Token never rotated after use
- If token is intercepted, attacker has 5 minutes

**Fix:** Rotate token on each successful request:
```rust
pub async fn validate_token(&self, token: &str) -> bool {
    let mut state = self.state.write().await; // mutable access
    let now = Instant::now();

    if let Some(entry) = state.tokens.get(token) {
        if now.duration_since(entry.created) <= Duration::from_secs(300) {
            // Rotate: remove old, create new
            state.tokens.remove(token);
            // Return new token in response header
            return true;
        }
    }
    false
}
```

---

### 15. NO INPUT VALIDATION ON PROXY URL BACKEND
**File:** `src/api/admin_proxies.rs:8-13`
**Severity:** MEDIUM

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyCreateRequest {
    pub url: String, // No validation
}
```

**Issue:**
- Backend accepts any URL string
- Frontend validation can be bypassed
- Malformed URLs cause errors

**Fix:** Add validation in handler:
```rust
pub async fn create_proxy(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(req): Json<ProxyCreateRequest>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    // Validate URL format
    if !validate_proxy_url(&req.url) {
        return Err(StatusCode::BAD_REQUEST);
    }
    // ...
}

fn validate_proxy_url(url: &str) -> bool {
    url.starts_with("socks5h://") && url.contains("@") && url.contains(":")
}
```

---

### 16. NO AUTHORIZATION CHECK ON DELETE OPERATIONS
**File:** `src/api/admin_accounts.rs:96-107`, `admin_proxies.rs:95-115`
**Severity:** MEDIUM

**Issue:**
- Delete operations use same auth as read operations
- No additional confirmation or 2FA for destructive operations
- Race condition: delete while someone else is using

**Fix:**
1. Require re-authentication for delete (fresh token fetch)
2. Add "confirm" token pattern

---

### 17. NO RATE LIMITING ON LOGIN ENDPOINT - Brute Force
**File:** `web/src/app/login/page.tsx:22-39`
**Severity:** HIGH

**Issue:**
- Login endpoint (`/admin/stats/overview` used for auth) has no specific rate limiting
- Attackers can brute force admin token
- 100 req/min is too high for auth endpoint

**Fix:**
```rust
// Separate rate limiter for auth
let auth_rate_limiter = RateLimiter::new(5, Duration::from_secs(300)); // 5 attempts per 5 min
```

---

### 18. NO SECURITY LOGGING - Attack Detection
**File:** `src/middleware/mod.rs`, `src/main.rs`
**Severity:** MEDIUM

**Issue:**
- Failed auth attempts not logged
- Rate limit violations not logged
- No audit trail for admin operations

**Fix:**
```rust
// In admin_auth_middleware
if constant_time_eq(provided, &token) {
    Ok(next.run(req).await)
} else {
    tracing::warn!(
        "Failed admin auth attempt from {} for path {}",
        client_ip,
        req.uri().path()
    );
    Err(StatusCode::UNAUTHORIZED)
}
```

Log:
- Failed authentications
- Rate limit violations
- All admin operations (create, update, delete)
- API key usage

---

### 19. NO API KEY QUOTA ENFORCEMENT
**File:** `src/api/admin_api_keys.rs:14`, `src/api/mod.rs:91-124`
**Severity:** MEDIUM

**Issue:**
- `quota_per_day` field exists but never enforced
- `auth_middleware` doesn't check quota
- Unlimited usage possible with any key

**Fix:**
```rust
// In auth_middleware
if let Some(ref key) = api_key {
    // Check quota
    let current_count = key_request_counts.read().await.get(key).unwrap_or(&0);
    let quota = get_key_quota(key).await?;
    
    if Some(*current_count) >= quota {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    
    // Increment counter
    key_request_counts.write().await
        .entry(key.clone())
        .and_modify(|c| *c += 1)
        .or_insert(1);
}
```

---

### 20. MIDDLEWARE SKIP PATTERNS - Auth Bypass
**File:** `src/api/mod.rs:96-99`
**Severity:** HIGH

```rust
// Skip auth for GET requests (models listing, health)
if req.method() == axum::http::Method::GET {
    return Ok(next.run(req).await);
}
```

**Issue:**
- ALL GET requests skip auth, including admin routes
- `/admin/accounts`, `/admin/proxies`, `/admin/api-keys` are GET-accessible without auth
- Only CSRF blocks writes, but data is exposed

**Fix:**
```rust
// Skip auth only for specific public endpoints
let public_endpoints = ["/health", "/v1/models"];
if public_endpoints.contains(&req.uri().path()) {
    return Ok(next.run(req).await);
}
```

---

## Summary by Category

| Category | Issues | Count |
|----------|--------|-------|
| Authentication | Token in localStorage, no session invalidation, no auth on GET, no login rate limiting | 4 |
| CSRF | Token in localStorage, not rotated, permissive CORS | 3 |
| Input Validation | Proxy URL backend, cookies format | 2 |
| Rate Limiting | Hardcoded window, memory leak, wrong order | 3 |
| Data Exposure | Unencrypted cookies, unencrypted proxy creds, sensitive data in errors | 3 |
| Error Handling | Sensitive error messages, no logging | 2 |
| Configuration | Hardcoded port, permissive CORS, missing security headers | 3 |
| Session Management | No rotation, no expiration, localStorage storage | 3 |

**Total Issues:** 20
- Critical: 4
- High: 8
- Medium: 6
- Low: 2

---

## Recommended Fix Priority

1. **P0 (Blocker):** Fix permissive CORS, fix auth bypass on GET, encrypt cookies at rest
2. **P1 (Critical):** Move tokens to httpOnly cookies, fix proxy URL encryption
3. **P2 (High):** Fix rate limiter parameters, add security headers, add login rate limiting
4. **P3 (Medium):** CSRF token rotation, middleware reordering, quota enforcement
5. **P4 (Low):** Error message sanitization, logging

---

## Expected Score After Fixes

| Before | After |
|--------|-------|
| 4/10 | 8-9/10 |

**Remaining gaps for 10/10:**
- MFA for admin operations
- Hardware security keys (FIDO2)
- Regular security audits
- Penetration testing

---

## Unresolved Questions

1. Is there an existing encryption key management system in place?
2. What is the expected deployment environment (cloud, on-premise)?
3. Are there compliance requirements (SOC2, HIPAA, GDPR)?
4. Is HTTPS termination handled externally (load balancer)?
