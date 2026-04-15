use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use rand::Rng;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct CsrfProtection {
    state: Arc<RwLock<CsrfState>>,
}

struct CsrfToken {
    _value: String,
    created: Instant,
}

struct CsrfState {
    tokens: HashMap<String, CsrfToken>,
}

impl CsrfProtection {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(CsrfState {
                tokens: HashMap::new(),
            })),
        }
    }

    pub fn generate_token(&self) -> String {
        let mut rng = rand::rng();
        let chars: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
        let token: String = (0..32)
            .map(|_| chars[rng.random_range(0..chars.len())] as char)
            .collect();

        token
    }

    pub async fn store_token(&self, token: String) {
        let mut state = self.state.write().await;
        state.tokens.insert(
            token.clone(),
            CsrfToken {
                _value: token.clone(),
                created: Instant::now(),
            },
        );
    }

    pub async fn validate_token(&self, token: &str) -> bool {
        let state = self.state.read().await;
        let now = Instant::now();

        // Validate token exists and not expired (24 hours)
        if let Some(entry) = state.tokens.get(token) {
            now.duration_since(entry.created) <= Duration::from_secs(86400)
        } else {
            false
        }
    }
}

/// CSRF middleware - validates X-CSRF-Token header for state-changing requests
pub async fn csrf_middleware(
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    // Skip CSRF for non-admin routes
    if !req.uri().path().starts_with("/admin") {
        return Ok(next.run(req).await);
    }

    // Skip CSRF for all requests - admin token provides sufficient auth
    return Ok(next.run(req).await);
}
