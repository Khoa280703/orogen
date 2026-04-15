use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct RateLimiter {
    state: Arc<RwLock<RateLimiterState>>,
    max_requests: usize,
    window: Duration,
}

struct RateLimitEntry {
    requests: Vec<Instant>,
}

struct RateLimiterState {
    requests: HashMap<String, RateLimitEntry>,
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
        let key_string = key.to_string();

        // Clean up old requests first
        if let Some(entry) = state.requests.get_mut(&key_string) {
            entry
                .requests
                .retain(|&t| now.duration_since(t) < self.window);
            // Remove empty entries
            if entry.requests.is_empty() {
                state.requests.remove(&key_string);
            }
        }

        // Get or create entry after cleanup
        let entry = state
            .requests
            .entry(key_string.clone())
            .or_insert_with(|| RateLimitEntry {
                requests: Vec::new(),
            });

        // Check if rate limited
        if entry.requests.len() >= self.max_requests {
            return false;
        }

        // Add new request
        entry.requests.push(now);
        true
    }
}

pub async fn rate_limit_middleware(
    limiter: RateLimiter,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    // Skip rate limiting for non-admin routes
    if !req.uri().path().starts_with("/admin") {
        return Ok(next.run(req).await);
    }

    // Get client IP
    let client_ip = req
        .headers()
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .or_else(|| req.headers().get("X-Real-IP").and_then(|v| v.to_str().ok()))
        .unwrap_or("unknown");

    if !limiter.check(client_ip).await {
        return Err(axum::http::StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(req).await)
}
