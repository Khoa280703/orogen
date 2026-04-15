pub mod admin_accounts;
pub mod admin_api_keys;
pub mod admin_conversations;
pub mod admin_health;
pub mod admin_images;
pub mod admin_models;
pub mod admin_payments;
pub mod admin_plans;
pub mod admin_providers;
pub mod admin_proxies;
pub mod admin_revenue;
pub mod admin_stats;
pub mod admin_users;
pub mod chat_completions;
pub mod consumer_api_support;
pub mod consumer_chat;
pub mod consumer_images;
pub mod consumer_videos;
pub mod image_generations;
pub mod models;
pub mod plan_enforcement;
pub mod user;
pub mod user_auth;
pub mod user_billing;
pub mod video_generations;

use axum::Json;
use axum::Router;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use axum::routing::{delete, get, post, put};

use crate::AppState;
use crate::api::chat_completions::ApiKey;
use crate::middleware::csrf::csrf_middleware;
use crate::middleware::rate_limiter;

pub fn router(state: AppState) -> Router {
    let auth_state = state.clone();
    let admin_token = state.config.admin_token.clone();

    let api_routes = Router::new()
        .route("/models", get(models::list_models))
        .route(
            "/chat/completions",
            post(chat_completions::chat_completions),
        )
        .route(
            "/images/generations",
            post(image_generations::generate_images),
        )
        .route(
            "/videos/generations",
            post(video_generations::generate_videos),
        );

    let admin_routes = Router::new()
        // CSRF token endpoint
        .route("/csrf-token", get(get_csrf_token))
        // Proxies
        .route("/proxies", get(admin_proxies::list_proxies))
        .route("/proxies", post(admin_proxies::create_proxy))
        .route("/proxies/:id", put(admin_proxies::update_proxy))
        .route("/proxies/:id", delete(admin_proxies::delete_proxy))
        // Accounts
        .route("/accounts", get(admin_accounts::list_accounts))
        .route("/accounts", post(admin_accounts::create_account))
        .route("/accounts/:id", put(admin_accounts::update_account))
        .route("/accounts/:id", delete(admin_accounts::delete_account))
        .route(
            "/accounts/:id/open-login-browser",
            post(admin_accounts::open_login_browser),
        )
        .route("/accounts/:id/sync-profile", post(admin_accounts::sync_profile))
        // API Keys
        .route("/api-keys", get(admin_api_keys::list_api_keys))
        .route("/api-keys", post(admin_api_keys::create_api_key))
        .route("/api-keys/:id", put(admin_api_keys::update_api_key))
        .route("/api-keys/:id", delete(admin_api_keys::delete_api_key))
        // Stats
        .route("/stats/overview", get(admin_stats::get_stats_overview))
        .route("/stats/usage", get(admin_stats::get_daily_usage))
        .route("/stats/logs", get(admin_stats::get_usage_logs))
        // Payments
        .route("/payments", get(admin_payments::list_payments))
        .route(
            "/payments/:id/approve",
            put(admin_payments::approve_payment),
        )
        .route("/payments/:id/reject", put(admin_payments::reject_payment))
        // Users
        .route("/users", get(admin_users::list_users))
        .route("/users/:id", get(admin_users::get_user_detail))
        .route("/users/:id", put(admin_users::update_user))
        // Consumer activity
        .route("/conversations", get(admin_conversations::list_conversations))
        .route(
            "/conversations/:id",
            get(admin_conversations::get_conversation_detail),
        )
        .route(
            "/conversations/:id",
            delete(admin_conversations::delete_conversation),
        )
        .route("/images", get(admin_images::list_images))
        .route("/images/:id", get(admin_images::get_image_detail))
        .route("/images/:id", delete(admin_images::delete_image))
        // Providers
        .route("/providers", get(admin_providers::list_all_providers))
        .route("/providers", post(admin_providers::create_new_provider))
        .route(
            "/providers/:id",
            put(admin_providers::update_existing_provider),
        )
        // Models
        .route("/models", get(admin_models::list_all_models))
        .route("/models", post(admin_models::create_new_model))
        .route("/models/:id", put(admin_models::update_existing_model))
        .route("/models/:id", delete(admin_models::delete_model))
        // Plans
        .route("/plans", get(admin_plans::list_plans))
        .route("/plans", post(admin_plans::create_plan))
        .route("/plans/:id", put(admin_plans::update_plan))
        .route("/plans/:id", delete(admin_plans::delete_plan))
        .route("/plans/:id/models", get(admin_plans::list_plan_models))
        .route("/plans/:id/models", post(admin_plans::add_models_to_plan))
        .route("/plans/:id/models", put(admin_plans::set_all_plan_models))
        .route(
            "/plans/:id/models/:model_id",
            delete(admin_plans::remove_model_from_plan_endpoint),
        )
        // Revenue
        .route(
            "/revenue/overview",
            get(admin_revenue::get_revenue_overview),
        )
        .route("/revenue/daily", get(admin_revenue::get_daily_revenue))
        .route(
            "/revenue/methods",
            get(admin_revenue::get_revenue_by_method),
        )
        // Health
        .route("/health", get(admin_health::get_health_overview));
    let consumer_routes = Router::new()
        .route("/chat/conversations", post(consumer_chat::create_conversation))
        .route("/chat/conversations", get(consumer_chat::list_conversations))
        .route("/chat/conversations/:id", get(consumer_chat::get_conversation))
        .route("/chat/conversations/:id", delete(consumer_chat::delete_conversation))
        .route("/chat/conversations/:id/messages", post(consumer_chat::send_message))
        .route("/images/generate", post(consumer_images::generate_images))
        .route("/videos/generate", post(consumer_videos::generate_videos))
        .route("/images/history", get(consumer_images::list_history))
        .route("/images/history/:id", get(consumer_images::get_generation));

    let rate_limiter = rate_limiter::RateLimiter::new(100, std::time::Duration::from_secs(60));

    let csrf_protection = state.config.csrf_protection.clone();

    // Auth routes (public - no auth required)
    let auth_routes = Router::new().route("/google", post(user_auth::handle_google_auth));

    let user_routes = Router::new()
        .route("/me", get(user::get_user_profile))
        .route("/keys", get(user::list_user_api_keys))
        .route("/keys", post(user::create_user_api_key))
        .route("/keys/:id", delete(user::revoke_user_api_key))
        .route("/usage", get(user::get_user_usage))
        .route("/billing", get(user::get_user_billing))
        .route("/topup/manual", post(user_billing::create_manual_topup))
        .route("/topup/crypto", post(user_billing::create_crypto_topup));

    let app = Router::new()
        .route("/", get(health_check))
        .route("/api/plans", get(list_public_plans))
        .nest("/api", consumer_routes)
        .nest("/auth", auth_routes)
        .nest("/v1", api_routes)
        .nest("/user", user_routes)
        .nest("/admin", admin_routes)
        // Rate limiting FIRST to block attackers before auth processing
        .layer(axum::middleware::from_fn(move |req, next| {
            rate_limiter::rate_limit_middleware(rate_limiter.clone(), req, next)
        }))
        // CSRF middleware BEFORE admin auth (so body is not consumed)
        .layer(axum::middleware::from_fn(
            move |req: Request<axum::body::Body>, next| {
                let mut req = req;
                req.extensions_mut().insert(csrf_protection.clone());
                csrf_middleware(req, next)
            },
        ))
        // JWT middleware for /user/* routes
        .layer(axum::middleware::from_fn(
            crate::middleware::jwt_auth::jwt_middleware,
        ))
        .layer(axum::middleware::from_fn(move |req, next| {
            auth_middleware(auth_state.clone(), req, next)
        }))
        .layer(axum::middleware::from_fn(move |req, next| {
            admin_auth_middleware(admin_token.clone(), req, next)
        }));

    app.with_state(state)
}

async fn get_csrf_token(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Json<serde_json::Value> {
    let csrf_token = state.config.csrf_protection.generate_token();
    state
        .config
        .csrf_protection
        .store_token(csrf_token.clone())
        .await;
    Json(serde_json::json!({ "token": csrf_token }))
}

async fn health_check() -> &'static str {
    r#"{"status":"ok","service":"grok-local"}"#
}

async fn list_public_plans(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, String)> {
    let db = &state.db;

    match crate::db::plans::list_plans(db).await {
        Ok(plans) => {
            let json = serde_json::to_value(&plans).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Serialize error: {}", e),
                )
            })?;
            Ok(axum::Json(json))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )),
    }
}

async fn auth_middleware(
    state: AppState,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = req.uri().path();
    if path.starts_with("/admin") {
        return Ok(next.run(req).await);
    }

    // Skip auth for /user/* routes (they use JWT auth instead)
    if path.starts_with("/user") {
        return Ok(next.run(req).await);
    }

    // Consumer routes use JWT middleware, not API keys.
    if path.starts_with("/api/chat") || path.starts_with("/api/images") {
        return Ok(next.run(req).await);
    }

    // Skip auth only for specific public endpoints (not all GET requests)
    let public_paths = ["/", "/health", "/v1/models", "/auth", "/api/plans"];
    if public_paths
        .iter()
        .any(|p| path == *p || path.starts_with(&format!("{}/", p)))
    {
        return Ok(next.run(req).await);
    }

    // Extract API key from Authorization header
    let api_key = extract_api_key(req.headers());

    // For POST requests, inject API key as Extension
    if req.method() == axum::http::Method::POST {
        if let Some(ref key) = api_key {
            let is_valid = {
                let keys = state.api_keys.read().await;
                keys.contains(key)
            };
            if !is_valid {
                return Err(StatusCode::UNAUTHORIZED);
            }
            req.extensions_mut().insert(ApiKey(key.clone()));
        } else if !state.api_keys.read().await.is_empty() {
            return Err(StatusCode::UNAUTHORIZED);
        } else {
            req.extensions_mut().insert(ApiKey(String::new()));
        }
    }

    Ok(next.run(req).await)
}

fn extract_api_key(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string())
        .or_else(|| {
            headers
                .get("x-api-key")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        })
}

async fn admin_auth_middleware(
    admin_token: Option<String>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip auth for non-admin routes
    if !req.uri().path().starts_with("/admin") {
        return Ok(next.run(req).await);
    }

    // Skip if no admin token configured
    let Some(token) = admin_token else {
        return Ok(next.run(req).await);
    };

    // Extract client IP for logging
    let client_ip = req
        .headers()
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next().map(|s| s.trim()))
        .or_else(|| req.headers().get("X-Real-IP").and_then(|v| v.to_str().ok()))
        .unwrap_or("unknown");

    let auth = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok());

    match auth {
        Some(val) => {
            let provided = val.strip_prefix("Bearer ").unwrap_or(val);
            // Constant-time comparison to prevent timing attacks
            if constant_time_eq(provided, &token) {
                Ok(next.run(req).await)
            } else {
                // Log failed auth attempt for security monitoring
                tracing::warn!(
                    client_ip = client_ip,
                    path = %req.uri().path(),
                    "Failed admin authentication attempt"
                );
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        _ => {
            tracing::warn!(
                client_ip = client_ip,
                path = %req.uri().path(),
                "Missing authorization header"
            );
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

/// Constant-time string comparison to prevent timing attacks
fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let mut result: u8 = 0;
    for (x, y) in a_bytes.iter().zip(b_bytes.iter()) {
        result |= x ^ y;
    }
    result == 0
}
