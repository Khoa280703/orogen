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
pub mod request_orchestrator;
pub mod responses;
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
        .route("/responses", post(responses::create_response))
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
            "/accounts/:id/usage",
            get(admin_accounts::get_account_usage),
        )
        .route(
            "/accounts/:id/open-login-browser",
            post(admin_accounts::open_login_browser),
        )
        .route(
            "/accounts/:id/sync-profile",
            post(admin_accounts::sync_profile),
        )
        .route(
            "/accounts/:id/start-codex-login",
            post(admin_accounts::start_codex_login),
        )
        .route(
            "/accounts/:id/codex-login-status",
            get(admin_accounts::get_codex_login_status),
        )
        .route(
            "/accounts/:id/complete-codex-login",
            post(admin_accounts::submit_codex_manual_callback),
        )
        .route(
            "/accounts/:id/refresh-codex-token",
            post(admin_accounts::refresh_codex_account),
        )
        .route(
            "/accounts/codex-import/start",
            post(admin_accounts::start_codex_import_login),
        )
        .route(
            "/accounts/codex-import/:session_id",
            get(admin_accounts::get_codex_login_status_by_session_id),
        )
        .route(
            "/accounts/codex-import/:session_id/complete",
            post(admin_accounts::submit_codex_manual_callback_by_session_id),
        )
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
        .route(
            "/conversations",
            get(admin_conversations::list_conversations),
        )
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
        .route(
            "/chat/conversations",
            post(consumer_chat::create_conversation),
        )
        .route(
            "/chat/conversations",
            get(consumer_chat::list_conversations),
        )
        .route(
            "/chat/conversations/:id",
            get(consumer_chat::get_conversation),
        )
        .route(
            "/chat/conversations/:id",
            delete(consumer_chat::delete_conversation),
        )
        .route(
            "/chat/conversations/:id/messages",
            post(consumer_chat::send_message),
        )
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
    if req.uri().path() == "/admin/accounts/codex/callback" {
        return Ok(next.run(req).await);
    }

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

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use std::panic::AssertUnwindSafe;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use std::{fs, time::Duration};

    use axum::Router;
    use axum::body::{Body, to_bytes};
    use axum::extract::State as AxumState;
    use axum::http::{Method, Request, StatusCode, header};
    use axum::response::IntoResponse;
    use axum::routing::{any, post};
    use futures::FutureExt;
    use futures::future::BoxFuture;
    use serde_json::{Value, json};
    use sqlx::postgres::PgConnection;
    use sqlx::{Connection, Executor};
    use tokio::net::TcpListener;
    use tokio::process::Command;
    use tokio::sync::RwLock;
    use tokio::task::JoinHandle;
    use tokio::time::timeout;
    use tower::ServiceExt;

    use crate::AppState;
    use crate::account::pool::AccountPool;
    use crate::account::types::{
        AUTH_MODE_CODEX_OAUTH, CREDENTIAL_TYPE_CODEX_OAUTH_TOKENS, ROUTING_STATE_AUTH_INVALID,
        ROUTING_STATE_COOLING_DOWN, ROUTING_STATE_HEALTHY,
    };
    use crate::config::AppConfig;
    use crate::db::account_sessions::SESSION_STATUS_EXPIRED;
    use crate::grok::client::GrokClient;
    use crate::providers::ProviderRegistry;
    use crate::services::codex_client::CodexClient;
    use crate::services::codex_oauth::CodexLoginSession;

    struct SeededRoutes {
        allowed_model_slug: String,
        blocked_model_slug: String,
        api_key: String,
    }

    struct TestDatabase {
        admin_url: String,
        database_name: String,
        database_url: String,
        pool: sqlx::PgPool,
    }

    impl TestDatabase {
        async fn create() -> Result<Self, String> {
            dotenvy::dotenv().ok();
            let base_database_url = std::env::var("DATABASE_URL")
                .map_err(|_| "DATABASE_URL is required for router integration tests".to_string())?;
            let admin_url = replace_database_name(&base_database_url, "postgres");
            let database_name = format!("duanai_test_{}", uuid::Uuid::new_v4().simple());
            let database_url = replace_database_name(&base_database_url, &database_name);

            let mut admin = match PgConnection::connect(&admin_url).await {
                Ok(connection) => connection,
                Err(error) => {
                    return Err(format!(
                        "router integration tests require admin Postgres access: {error}"
                    ));
                }
            };
            if let Err(error) = admin
                .execute(sqlx::query(&format!(
                    r#"CREATE DATABASE "{database_name}""#
                )))
                .await
            {
                return Err(format!(
                    "router integration tests require CREATE DATABASE privilege: {error}"
                ));
            }
            drop(admin);

            let bootstrap_pool = match crate::db::init_pool(&database_url).await {
                Ok(pool) => pool,
                Err(error) => {
                    cleanup_temp_database(&admin_url, &database_name).await?;
                    return Err(format!("failed to connect to temporary database: {error}"));
                }
            };
            if let Err(error) =
                sqlx::raw_sql(include_str!("../../migrations/001_initial_schema.sql"))
                    .execute(&bootstrap_pool)
                    .await
            {
                bootstrap_pool.close().await;
                cleanup_temp_database(&admin_url, &database_name).await?;
                return Err(format!("failed to apply initial schema: {error}"));
            }
            bootstrap_pool.close().await;

            if let Err(error) = crate::db::migrate::run_migrations(&database_url).await {
                cleanup_temp_database(&admin_url, &database_name).await?;
                return Err(format!("failed to run migrations: {error}"));
            }

            let pool = match crate::db::init_pool(&database_url).await {
                Ok(pool) => pool,
                Err(error) => {
                    cleanup_temp_database(&admin_url, &database_name).await?;
                    return Err(format!(
                        "failed to reconnect to temporary database: {error}"
                    ));
                }
            };

            Ok(Self {
                admin_url,
                database_name,
                database_url,
                pool,
            })
        }

        async fn cleanup(self) {
            self.pool.close().await;
            cleanup_temp_database(&self.admin_url, &self.database_name)
                .await
                .expect("failed to drop temporary database");
        }
    }

    struct TestHarness {
        app: Router,
        db: TestDatabase,
        seed: SeededRoutes,
    }

    impl TestHarness {
        async fn try_new() -> Result<Self, String> {
            Self::try_new_with_codex_upstream(None).await
        }

        async fn try_new_with_codex_upstream(
            codex_upstream_base_url: Option<String>,
        ) -> Result<Self, String> {
            let db = TestDatabase::create().await?;
            let seed = seed_router_fixtures(&db.pool).await;
            let app = super::router(build_test_state(&db, &seed, codex_upstream_base_url).await);
            Ok(Self { app, db, seed })
        }

        async fn cleanup(self) {
            drop(self.app);
            self.db.cleanup().await;
        }
    }

    async fn with_test_harness<F>(test: F)
    where
        F: for<'a> FnOnce(&'a TestHarness) -> BoxFuture<'a, ()>,
    {
        let harness = TestHarness::try_new().await.expect(
            "router integration tests require DATABASE_URL and CREATE/DROP DATABASE access",
        );

        let outcome = AssertUnwindSafe(test(&harness)).catch_unwind().await;
        harness.cleanup().await;

        if let Err(panic) = outcome {
            std::panic::resume_unwind(panic);
        }
    }

    async fn with_test_harness_and_codex_upstream<F>(codex_upstream_base_url: String, test: F)
    where
        F: for<'a> FnOnce(&'a TestHarness) -> BoxFuture<'a, ()>,
    {
        let harness = TestHarness::try_new_with_codex_upstream(Some(codex_upstream_base_url))
            .await
            .expect(
                "router integration tests require DATABASE_URL and CREATE/DROP DATABASE access",
            );

        let outcome = AssertUnwindSafe(test(&harness)).catch_unwind().await;
        harness.cleanup().await;

        if let Err(panic) = outcome {
            std::panic::resume_unwind(panic);
        }
    }

    async fn build_test_state(
        db: &TestDatabase,
        seed: &SeededRoutes,
        codex_upstream_base_url: Option<String>,
    ) -> AppState {
        let mut config = AppConfig::default();
        config.database_url = Some(db.database_url.clone());
        config.default_model = seed.allowed_model_slug.clone();
        if let Some(url) = codex_upstream_base_url {
            config.codex_upstream_base_url = url;
        }

        let grok = GrokClient::new()
            .await
            .expect("failed to create Grok client");
        let providers = ProviderRegistry::new(
            grok.clone(),
            CodexClient::new(
                config.codex_upstream_base_url.clone(),
                config.codex_upstream_originator.clone(),
                config.codex_upstream_user_agent.clone(),
            ),
        );

        AppState {
            config: config.clone(),
            pool: AccountPool::new(db.pool.clone(), config, providers.clone()),
            grok,
            key_request_counts: Arc::new(RwLock::new(HashMap::new())),
            db: db.pool.clone(),
            api_keys: Arc::new(RwLock::new(HashSet::from([seed.api_key.clone()]))),
            providers,
            codex_login_sessions: Arc::new(
                RwLock::new(HashMap::<String, CodexLoginSession>::new()),
            ),
        }
    }

    async fn seed_router_fixtures(pool: &sqlx::PgPool) -> SeededRoutes {
        sqlx::query(
            r#"
            INSERT INTO providers (name, slug, active)
            VALUES ('Codex', 'codex', true)
            ON CONFLICT (slug) DO UPDATE SET active = EXCLUDED.active
            "#,
        )
        .execute(pool)
        .await
        .expect("failed to ensure codex provider");

        let plan_id: i32 = sqlx::query_scalar(
            r#"
            INSERT INTO plans (name, slug, active)
            VALUES ('Router Test Plan', 'router-test-plan', true)
            RETURNING id
            "#,
        )
        .fetch_one(pool)
        .await
        .expect("failed to create plan");

        let allowed_public_model_id: i32 = sqlx::query_scalar(
            r#"
            INSERT INTO public_models (slug, display_name, description, active)
            VALUES ('test-codex-allowed', 'test-codex-allowed', 'Allowed routed model', true)
            RETURNING id
            "#,
        )
        .fetch_one(pool)
        .await
        .expect("failed to create allowed public model");

        let blocked_public_model_id: i32 = sqlx::query_scalar(
            r#"
            INSERT INTO public_models (slug, display_name, description, active)
            VALUES ('test-codex-blocked', 'test-codex-blocked', 'Blocked routed model', true)
            RETURNING id
            "#,
        )
        .fetch_one(pool)
        .await
        .expect("failed to create blocked public model");

        sqlx::query(
            r#"
            INSERT INTO public_model_routes (
                public_model_id,
                provider_slug,
                upstream_model_slug,
                route_priority,
                active
            )
            VALUES
                ($1, 'codex', 'gpt-5.1', 0, true),
                ($2, 'codex', 'gpt-5', 0, true)
            "#,
        )
        .bind(allowed_public_model_id)
        .bind(blocked_public_model_id)
        .execute(pool)
        .await
        .expect("failed to create public model routes");

        sqlx::query(
            r#"
            INSERT INTO plan_public_models (plan_id, public_model_id)
            VALUES ($1, $2)
            "#,
        )
        .bind(plan_id)
        .bind(allowed_public_model_id)
        .execute(pool)
        .await
        .expect("failed to map plan to allowed public model");

        let api_key = "sk-router-test-key".to_string();
        sqlx::query(
            r#"
            INSERT INTO api_keys (key, label, active, quota_per_day, plan_id)
            VALUES ($1, 'router-test-key', true, NULL, $2)
            "#,
        )
        .bind(&api_key)
        .bind(plan_id)
        .execute(pool)
        .await
        .expect("failed to create API key");

        SeededRoutes {
            allowed_model_slug: "test-codex-allowed".to_string(),
            blocked_model_slug: "test-codex-blocked".to_string(),
            api_key,
        }
    }

    #[derive(Clone, Copy)]
    enum CodexPrimaryFailureMode {
        Success,
        RateLimited,
        Unauthorized,
        UpstreamTransient,
        PostStreamUnauthorized,
        PostStreamTruncated,
        PostStreamInvalidJson,
    }

    #[derive(Clone)]
    struct CodexStubState {
        primary_token: String,
        fallback_token: String,
        fallback_text: String,
        primary_failure_mode: CodexPrimaryFailureMode,
        hits: Arc<RwLock<Vec<String>>>,
    }

    struct CodexUpstreamStub {
        base_url: String,
        hits: Arc<RwLock<Vec<String>>>,
        server: JoinHandle<()>,
    }

    #[derive(Clone, Copy)]
    enum ProxyStubMode {
        CfBlocked,
        Success,
        PostStreamCfBlocked,
    }

    #[derive(Clone)]
    struct ProxyStubState {
        label: String,
        mode: ProxyStubMode,
        success_text: Option<String>,
        hits: Arc<RwLock<Vec<String>>>,
    }

    struct HttpProxyStub {
        url: String,
        hits: Arc<RwLock<Vec<String>>>,
        server: JoinHandle<()>,
    }

    impl CodexUpstreamStub {
        async fn spawn(primary_token: &str, fallback_token: &str, fallback_text: &str) -> Self {
            Self::spawn_with_primary_failure(
                primary_token,
                fallback_token,
                fallback_text,
                CodexPrimaryFailureMode::RateLimited,
            )
            .await
        }

        async fn spawn_with_primary_failure(
            primary_token: &str,
            fallback_token: &str,
            fallback_text: &str,
            primary_failure_mode: CodexPrimaryFailureMode,
        ) -> Self {
            let hits = Arc::new(RwLock::new(Vec::new()));
            let state = CodexStubState {
                primary_token: primary_token.to_string(),
                fallback_token: fallback_token.to_string(),
                fallback_text: fallback_text.to_string(),
                primary_failure_mode,
                hits: hits.clone(),
            };

            let app = Router::new()
                .route("/", post(codex_upstream_stub_handler))
                .with_state(state);
            let listener = TcpListener::bind("127.0.0.1:0")
                .await
                .expect("failed to bind Codex upstream stub");
            let addr = listener
                .local_addr()
                .expect("failed to read Codex upstream stub address");
            let server = tokio::spawn(async move {
                axum::serve(listener, app)
                    .await
                    .expect("Codex upstream stub should stay healthy during test");
            });

            Self {
                base_url: format!("http://{addr}"),
                hits,
                server,
            }
        }

        async fn recorded_tokens(&self) -> Vec<String> {
            self.hits.read().await.clone()
        }

        fn shutdown(self) {
            self.server.abort();
        }
    }

    impl HttpProxyStub {
        async fn spawn(
            label: &str,
            mode: ProxyStubMode,
            success_text: Option<&str>,
            hits: Arc<RwLock<Vec<String>>>,
        ) -> Self {
            let state = ProxyStubState {
                label: label.to_string(),
                mode,
                success_text: success_text.map(str::to_string),
                hits: hits.clone(),
            };

            let app = Router::new()
                .route("/", any(proxy_stub_handler))
                .fallback(any(proxy_stub_handler))
                .with_state(state);
            let listener = TcpListener::bind("127.0.0.1:0")
                .await
                .expect("failed to bind proxy stub");
            let addr = listener
                .local_addr()
                .expect("failed to read proxy stub address");
            let server = tokio::spawn(async move {
                axum::serve(listener, app)
                    .await
                    .expect("proxy stub should stay healthy during test");
            });

            Self {
                url: format!("http://{addr}"),
                hits,
                server,
            }
        }

        async fn recorded_hits(&self) -> Vec<String> {
            self.hits.read().await.clone()
        }

        fn shutdown(self) {
            self.server.abort();
        }
    }

    async fn codex_upstream_stub_handler(
        AxumState(state): AxumState<CodexStubState>,
        headers: axum::http::HeaderMap,
    ) -> impl IntoResponse {
        let token = headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .unwrap_or_default()
            .to_string();
        state.hits.write().await.push(token.clone());

        if token == state.primary_token {
            return match state.primary_failure_mode {
                CodexPrimaryFailureMode::Success => (
                    [(header::CONTENT_TYPE, "text/event-stream")],
                    codex_success_sse(&state.fallback_text),
                )
                    .into_response(),
                CodexPrimaryFailureMode::RateLimited => (
                    StatusCode::TOO_MANY_REQUESTS,
                    "Too many requests from Codex upstream",
                )
                    .into_response(),
                CodexPrimaryFailureMode::Unauthorized => {
                    (StatusCode::UNAUTHORIZED, "Unauthorized").into_response()
                }
                CodexPrimaryFailureMode::UpstreamTransient => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Temporary Codex upstream failure",
                )
                    .into_response(),
                CodexPrimaryFailureMode::PostStreamUnauthorized => (
                    [(header::CONTENT_TYPE, "text/event-stream")],
                    codex_failed_sse("partial token before auth failure", "invalid token"),
                )
                    .into_response(),
                CodexPrimaryFailureMode::PostStreamTruncated => (
                    [(header::CONTENT_TYPE, "text/event-stream")],
                    codex_partial_sse("partial token before truncated stream"),
                )
                    .into_response(),
                CodexPrimaryFailureMode::PostStreamInvalidJson => (
                    [(header::CONTENT_TYPE, "text/event-stream")],
                    codex_invalid_json_sse("partial token before parser failure"),
                )
                    .into_response(),
            };
        }

        if token == state.fallback_token {
            return (
                [(header::CONTENT_TYPE, "text/event-stream")],
                codex_success_sse(&state.fallback_text),
            )
                .into_response();
        }

        (StatusCode::UNAUTHORIZED, "Unauthorized").into_response()
    }

    async fn proxy_stub_handler(
        AxumState(state): AxumState<ProxyStubState>,
        _request: Request<Body>,
    ) -> impl IntoResponse {
        state.hits.write().await.push(state.label.clone());

        match state.mode {
            ProxyStubMode::CfBlocked => (
                StatusCode::FORBIDDEN,
                "Cloudflare blocked via proxy cf-ray test",
            )
                .into_response(),
            ProxyStubMode::Success => (
                [(header::CONTENT_TYPE, "text/event-stream")],
                codex_success_sse(
                    state
                        .success_text
                        .as_deref()
                        .unwrap_or("proxy success response"),
                ),
            )
                .into_response(),
            ProxyStubMode::PostStreamCfBlocked => (
                [(header::CONTENT_TYPE, "text/event-stream")],
                codex_failed_sse(
                    "partial token before proxy failure",
                    "Cloudflare blocked cf-ray",
                ),
            )
                .into_response(),
        }
    }

    fn codex_success_sse(text: &str) -> String {
        format!(
            concat!(
                "event: response.output_text.delta\n",
                "data: {{\"type\":\"response.output_text.delta\",\"delta\":{text:?}}}\n\n",
                "event: response.completed\n",
                "data: {{\"type\":\"response.completed\",\"response\":{{\"output\":[{{\"type\":\"message\",\"content\":[{{\"type\":\"output_text\",\"text\":{text:?}}}]}}]}}}}\n\n"
            ),
            text = text
        )
    }

    fn codex_failed_sse(text: &str, error_message: &str) -> String {
        format!(
            concat!(
                "event: response.output_text.delta\n",
                "data: {{\"type\":\"response.output_text.delta\",\"delta\":{text:?}}}\n\n",
                "event: response.failed\n",
                "data: {{\"type\":\"response.failed\",\"error\":{{\"message\":{error_message:?}}}}}\n\n"
            ),
            text = text,
            error_message = error_message
        )
    }

    fn codex_partial_sse(text: &str) -> String {
        format!(
            concat!(
                "event: response.output_text.delta\n",
                "data: {{\"type\":\"response.output_text.delta\",\"delta\":{text:?}}}\n\n"
            ),
            text = text
        )
    }

    fn codex_invalid_json_sse(text: &str) -> String {
        format!(
            concat!(
                "event: response.output_text.delta\n",
                "data: {{\"type\":\"response.output_text.delta\",\"delta\":{text:?}}}\n\n",
                "event: response.failed\n",
                "data: {{oops-invalid-json}}\n\n"
            ),
            text = text
        )
    }

    async fn seed_codex_runtime_account(
        pool: &sqlx::PgPool,
        name: &str,
        access_token: &str,
        _mark_healthy: bool,
    ) -> i32 {
        let account_id = crate::db::accounts::create_account(
            pool,
            name,
            "codex",
            &json!({}),
            None,
            None,
            Some(AUTH_MODE_CODEX_OAUTH),
        )
        .await
        .expect("failed to create Codex runtime account");

        crate::db::account_credentials::upsert_account_credential(
            pool,
            account_id,
            CREDENTIAL_TYPE_CODEX_OAUTH_TOKENS,
            &json!({
                "access_token": access_token,
                "refresh_token": format!("refresh-{access_token}"),
                "expires_at": (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339(),
            }),
        )
        .await
        .expect("failed to store Codex runtime credential");

        account_id
    }

    async fn seed_proxy(pool: &sqlx::PgPool, url: &str, label: &str) -> i32 {
        crate::db::proxies::create_proxy(pool, url, Some(label))
            .await
            .expect("failed to create test proxy")
    }

    async fn send_request_raw_with_content_type(
        app: &Router,
        method: Method,
        path: &str,
        body: Option<Value>,
        bearer_token: Option<&str>,
    ) -> (StatusCode, Option<String>, String) {
        let payload = body.map(|value| value.to_string()).unwrap_or_default();
        let mut builder = Request::builder().method(method).uri(path);
        if let Some(token) = bearer_token {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
        }
        if !payload.is_empty() {
            builder = builder.header(header::CONTENT_TYPE, "application/json");
        }

        let response = app
            .clone()
            .oneshot(
                builder
                    .body(if payload.is_empty() {
                        Body::empty()
                    } else {
                        Body::from(payload)
                    })
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        let status = response.status();
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(str::to_string);
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("failed to read response body");
        (
            status,
            content_type,
            String::from_utf8(bytes.to_vec()).expect("response body should be valid UTF-8"),
        )
    }

    async fn send_request_raw(
        app: &Router,
        method: Method,
        path: &str,
        body: Option<Value>,
        bearer_token: Option<&str>,
    ) -> (StatusCode, String) {
        let (status, _, body) =
            send_request_raw_with_content_type(app, method, path, body, bearer_token).await;
        (status, body)
    }

    async fn send_request(
        app: &Router,
        method: Method,
        path: &str,
        body: Option<Value>,
        bearer_token: Option<&str>,
    ) -> (StatusCode, Value) {
        let (status, body) = send_request_raw(app, method, path, body, bearer_token).await;
        let parsed = if body.trim().is_empty() {
            Value::Null
        } else {
            serde_json::from_str(&body).expect("response body should be valid JSON")
        };

        (status, parsed)
    }

    async fn spawn_router_server(app: Router) -> (String, JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind router test server");
        let addr = listener
            .local_addr()
            .expect("failed to read router test server address");
        let server = tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("router test server should stay healthy during test");
        });

        (format!("http://{addr}/v1"), server)
    }

    fn write_codex_cli_smoke_home(base_url: &str, api_key: &str, model: &str) -> PathBuf {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("codex-cli-smoke")
            .join(uuid::Uuid::new_v4().simple().to_string());
        let codex_dir = root.join(".codex");
        fs::create_dir_all(&codex_dir).expect("failed to create Codex smoke home");

        fs::write(
            codex_dir.join("config.toml"),
            format!(
                concat!(
                    "model = \"{model}\"\n",
                    "model_provider = \"duanai\"\n\n",
                    "[model_providers.duanai]\n",
                    "name = \"DuanAI\"\n",
                    "base_url = \"{base_url}\"\n",
                    "wire_api = \"responses\"\n"
                ),
                model = model,
                base_url = base_url
            ),
        )
        .expect("failed to write Codex smoke config");

        fs::write(
            codex_dir.join("auth.json"),
            format!(
                concat!("{{\n", "  \"OPENAI_API_KEY\": \"{api_key}\"\n", "}}\n"),
                api_key = api_key
            ),
        )
        .expect("failed to write Codex smoke auth");

        root
    }

    async fn run_codex_exec_smoke(home_dir: &Path, model: &str) -> Result<String, String> {
        let output_path = home_dir.join("last-message.txt");
        let command = Command::new("codex")
            .env("HOME", home_dir)
            .env("OPENAI_API_KEY", "sk-router-test-key")
            .arg("exec")
            .arg("--skip-git-repo-check")
            .arg("-C")
            .arg("/tmp")
            .arg("-m")
            .arg(model)
            .arg("-o")
            .arg(&output_path)
            .arg("Reply with exactly gateway smoke test ok and nothing else.")
            .output();

        let output = timeout(Duration::from_secs(90), command)
            .await
            .map_err(|_| "codex exec smoke test timed out".to_string())?
            .map_err(|error| format!("failed to run codex exec smoke test: {error}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !output.status.success() {
            return Err(format!(
                "codex exec smoke test failed with status {}.\nstdout:\n{}\nstderr:\n{}",
                output.status,
                stdout.trim(),
                stderr.trim()
            ));
        }

        fs::read_to_string(&output_path)
            .map_err(|error| format!("failed to read codex exec output file: {error}"))
    }

    fn catalog_ids(body: &Value) -> Vec<String> {
        body["data"]
            .as_array()
            .expect("catalog data should be an array")
            .iter()
            .filter_map(|entry| entry["id"].as_str().map(str::to_string))
            .collect()
    }

    fn assert_substrings_in_order(body: &str, expected: &[&str]) {
        let mut cursor = 0usize;
        for needle in expected {
            let found = body[cursor..].find(needle).unwrap_or_else(|| {
                panic!("missing substring in order check: {needle}\nbody={body}")
            });
            cursor += found + needle.len();
        }
    }

    fn replace_database_name(database_url: &str, database_name: &str) -> String {
        let (base, query) = database_url
            .split_once('?')
            .map_or((database_url, None), |(base, query)| (base, Some(query)));
        let (prefix, _) = base
            .rsplit_once('/')
            .expect("database URL should contain a database name");

        match query {
            Some(query) => format!("{prefix}/{database_name}?{query}"),
            None => format!("{prefix}/{database_name}"),
        }
    }

    async fn cleanup_temp_database(admin_url: &str, database_name: &str) -> Result<(), String> {
        let mut admin = PgConnection::connect(admin_url).await.map_err(|error| {
            format!("failed to reconnect to admin database for cleanup: {error}")
        })?;
        admin
            .execute(sqlx::query(&format!(
                r#"
                SELECT pg_terminate_backend(pid)
                FROM pg_stat_activity
                WHERE datname = '{database_name}'
                  AND pid <> pg_backend_pid()
                "#,
                database_name = database_name
            )))
            .await
            .map_err(|error| format!("failed to terminate temporary database sessions: {error}"))?;
        admin
            .execute(sqlx::query(&format!(
                r#"DROP DATABASE IF EXISTS "{database_name}""#,
                database_name = database_name
            )))
            .await
            .map_err(|error| format!("failed to drop temporary database: {error}"))?;
        Ok(())
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL with CREATE/DROP DATABASE access"]
    async fn lists_public_and_plan_scoped_models_from_router() {
        with_test_harness(|harness| {
            Box::pin(async move {
                let (anonymous_status, anonymous_body) =
                    send_request(&harness.app, Method::GET, "/v1/models", None, None).await;
                assert_eq!(anonymous_status, StatusCode::OK);
                let anonymous_ids = catalog_ids(&anonymous_body);
                assert!(anonymous_ids.contains(&harness.seed.allowed_model_slug));
                assert!(anonymous_ids.contains(&harness.seed.blocked_model_slug));

                let (scoped_status, scoped_body) = send_request(
                    &harness.app,
                    Method::GET,
                    "/v1/models",
                    None,
                    Some(&harness.seed.api_key),
                )
                .await;
                assert_eq!(scoped_status, StatusCode::OK);
                let scoped_ids = catalog_ids(&scoped_body);
                assert!(scoped_ids.contains(&harness.seed.allowed_model_slug));
                assert!(!scoped_ids.contains(&harness.seed.blocked_model_slug));
            })
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL with CREATE/DROP DATABASE access"]
    async fn responses_route_reaches_no_accounts_boundary_for_allowed_model() {
        with_test_harness(|harness| {
            Box::pin(async move {
                let (status, body) = send_request(
                    &harness.app,
                    Method::POST,
                    "/v1/responses",
                    Some(json!({
                        "model": harness.seed.allowed_model_slug,
                        "input": "hello from routed codex test"
                    })),
                    Some(&harness.seed.api_key),
                )
                .await;

                assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
                assert_eq!(body["error"]["message"], "Service unavailable");
            })
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL with CREATE/DROP DATABASE access"]
    async fn responses_route_rejects_models_outside_the_assigned_plan() {
        with_test_harness(|harness| {
            Box::pin(async move {
                let (status, body) = send_request(
                    &harness.app,
                    Method::POST,
                    "/v1/responses",
                    Some(json!({
                        "model": harness.seed.blocked_model_slug,
                        "input": "this model should be blocked"
                    })),
                    Some(&harness.seed.api_key),
                )
                .await;

                assert_eq!(status, StatusCode::FORBIDDEN);
                assert_eq!(body["error"]["message"], "Model not available in your plan");
            })
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL with CREATE/DROP DATABASE access"]
    async fn responses_route_ignores_tools_but_rejects_image_payloads_before_upstream() {
        with_test_harness(|harness| {
            Box::pin(async move {
                let (tools_status, tools_body) = send_request(
                    &harness.app,
                    Method::POST,
                    "/v1/responses",
                    Some(json!({
                        "model": harness.seed.allowed_model_slug,
                        "input": "tool payload should fail",
                        "tools": [{ "type": "function", "name": "do_work" }]
                    })),
                    Some(&harness.seed.api_key),
                )
                .await;
                assert_eq!(tools_status, StatusCode::SERVICE_UNAVAILABLE);
                assert_eq!(
                    tools_body["error"]["message"],
                    "No upstream accounts configured"
                );

                let (image_status, image_body) = send_request(
                    &harness.app,
                    Method::POST,
                    "/v1/responses",
                    Some(json!({
                        "model": harness.seed.allowed_model_slug,
                        "input": {
                            "type": "input_image",
                            "image_url": "https://example.com/test.png"
                        }
                    })),
                    Some(&harness.seed.api_key),
                )
                .await;
                assert_eq!(image_status, StatusCode::BAD_REQUEST);
                assert_eq!(
                    image_body["error"]["message"],
                    "image input is not supported on /v1/responses yet"
                );
            })
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL with CREATE/DROP DATABASE access"]
    async fn responses_route_requires_a_valid_api_key_when_keys_are_configured() {
        with_test_harness(|harness| {
            Box::pin(async move {
                let request_body = json!({
                    "model": harness.seed.allowed_model_slug,
                    "input": "auth should fail before routing"
                });

                let (missing_status, _) = send_request(
                    &harness.app,
                    Method::POST,
                    "/v1/responses",
                    Some(request_body.clone()),
                    None,
                )
                .await;
                assert_eq!(missing_status, StatusCode::UNAUTHORIZED);

                let (invalid_status, _) = send_request(
                    &harness.app,
                    Method::POST,
                    "/v1/responses",
                    Some(request_body),
                    Some("sk-router-invalid"),
                )
                .await;
                assert_eq!(invalid_status, StatusCode::UNAUTHORIZED);
            })
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL with CREATE/DROP DATABASE access"]
    async fn chat_completions_route_reaches_no_accounts_boundary_for_allowed_model() {
        with_test_harness(|harness| {
            Box::pin(async move {
                let (status, body) = send_request(
                    &harness.app,
                    Method::POST,
                    "/v1/chat/completions",
                    Some(json!({
                        "model": harness.seed.allowed_model_slug,
                        "messages": [
                            { "role": "user", "content": "hello from chat route test" }
                        ]
                    })),
                    Some(&harness.seed.api_key),
                )
                .await;

                assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
                assert_eq!(body["error"]["message"], "Service unavailable");
            })
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL with CREATE/DROP DATABASE access"]
    async fn chat_completions_route_rejects_models_outside_the_assigned_plan() {
        with_test_harness(|harness| {
            Box::pin(async move {
                let (status, body) = send_request(
                    &harness.app,
                    Method::POST,
                    "/v1/chat/completions",
                    Some(json!({
                        "model": harness.seed.blocked_model_slug,
                        "messages": [
                            { "role": "user", "content": "this model should be blocked" }
                        ]
                    })),
                    Some(&harness.seed.api_key),
                )
                .await;

                assert_eq!(status, StatusCode::FORBIDDEN);
                assert_eq!(body["error"]["message"], "Model not available in your plan");
            })
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL with CREATE/DROP DATABASE access"]
    async fn chat_completions_route_rejects_tools_and_unsupported_message_payloads() {
        with_test_harness(|harness| Box::pin(async move {
            let (tools_status, tools_body) = send_request(
                &harness.app,
                Method::POST,
                "/v1/chat/completions",
                Some(json!({
                    "model": harness.seed.allowed_model_slug,
                    "messages": [
                        { "role": "user", "content": "tool payload should fail" }
                    ],
                    "tools": [{ "type": "function", "name": "do_work" }]
                })),
                Some(&harness.seed.api_key),
            )
            .await;
            assert_eq!(tools_status, StatusCode::BAD_REQUEST);
            assert_eq!(
                tools_body["error"]["message"],
                "tools are no longer supported on /v1/chat/completions"
            );

            let (legacy_status, legacy_body) = send_request(
                &harness.app,
                Method::POST,
                "/v1/chat/completions",
                Some(json!({
                    "model": harness.seed.allowed_model_slug,
                    "messages": [
                        {
                            "role": "assistant",
                            "content": "",
                            "tool_calls": [{ "id": "call_1", "type": "function" }]
                        }
                    ]
                })),
                Some(&harness.seed.api_key),
            )
            .await;
            assert_eq!(legacy_status, StatusCode::BAD_REQUEST);
            assert_eq!(
                legacy_body["error"]["message"],
                "tool and function calling payloads are no longer supported on /v1/chat/completions"
            );

            let (image_status, image_body) = send_request(
                &harness.app,
                Method::POST,
                "/v1/chat/completions",
                Some(json!({
                    "model": harness.seed.allowed_model_slug,
                    "messages": [
                        {
                            "role": "user",
                            "content": [
                                {
                                    "type": "input_image",
                                    "image_url": "https://example.com/test.png"
                                }
                            ]
                        }
                    ]
                })),
                Some(&harness.seed.api_key),
            )
            .await;
            assert_eq!(image_status, StatusCode::BAD_REQUEST);
            assert_eq!(
                image_body["error"]["message"],
                "image input is not supported on /v1/chat/completions yet"
            );
        }))
        .await;
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL with CREATE/DROP DATABASE access"]
    async fn chat_completions_route_requires_a_valid_api_key_when_keys_are_configured() {
        with_test_harness(|harness| {
            Box::pin(async move {
                let request_body = json!({
                    "model": harness.seed.allowed_model_slug,
                    "messages": [
                        { "role": "user", "content": "auth should fail before routing" }
                    ]
                });

                let (missing_status, _) = send_request(
                    &harness.app,
                    Method::POST,
                    "/v1/chat/completions",
                    Some(request_body.clone()),
                    None,
                )
                .await;
                assert_eq!(missing_status, StatusCode::UNAUTHORIZED);

                let (invalid_status, _) = send_request(
                    &harness.app,
                    Method::POST,
                    "/v1/chat/completions",
                    Some(request_body),
                    Some("sk-router-invalid"),
                )
                .await;
                assert_eq!(invalid_status, StatusCode::UNAUTHORIZED);
            })
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL with CREATE/DROP DATABASE access"]
    async fn chat_completions_stream_route_emits_sse_error_for_no_accounts() {
        with_test_harness(|harness| {
            Box::pin(async move {
                let (status, body) = send_request_raw(
                    &harness.app,
                    Method::POST,
                    "/v1/chat/completions",
                    Some(json!({
                        "model": harness.seed.allowed_model_slug,
                        "stream": true,
                        "messages": [
                            { "role": "user", "content": "stream route should hit no accounts" }
                        ]
                    })),
                    Some(&harness.seed.api_key),
                )
                .await;

                assert_eq!(status, StatusCode::OK);
                assert!(body.contains(r#""message":"No upstream accounts configured""#));
                assert!(!body.contains("[DONE]"));
            })
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL with CREATE/DROP DATABASE access"]
    async fn chat_completions_stream_route_still_rejects_models_outside_plan_before_sse() {
        with_test_harness(|harness| {
            Box::pin(async move {
                let (status, body) = send_request(
                    &harness.app,
                    Method::POST,
                    "/v1/chat/completions",
                    Some(json!({
                        "model": harness.seed.blocked_model_slug,
                        "stream": true,
                        "messages": [
                            { "role": "user", "content": "blocked stream model" }
                        ]
                    })),
                    Some(&harness.seed.api_key),
                )
                .await;

                assert_eq!(status, StatusCode::FORBIDDEN);
                assert_eq!(body["error"]["message"], "Model not available in your plan");
            })
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL with CREATE/DROP DATABASE access"]
    async fn responses_stream_route_emits_sse_failure_and_done_for_no_accounts() {
        with_test_harness(|harness| {
            Box::pin(async move {
                let (status, content_type, body) = send_request_raw_with_content_type(
                    &harness.app,
                    Method::POST,
                    "/v1/responses",
                    Some(json!({
                        "model": harness.seed.allowed_model_slug,
                        "stream": true,
                        "input": "responses stream should hit no accounts"
                    })),
                    Some(&harness.seed.api_key),
                )
                .await;

                assert_eq!(status, StatusCode::OK);
                assert_eq!(content_type.as_deref(), Some("text/event-stream"));
                assert!(body.contains("event: response.failed"));
                assert!(body.contains(r#""type":"response.failed""#));
                assert!(body.contains(r#""message":"No upstream accounts configured""#));
                assert!(body.contains("[DONE]"));
            })
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL with CREATE/DROP DATABASE access and local codex binary"]
    async fn codex_exec_can_use_gateway_responses_surface_end_to_end() {
        let upstream = CodexUpstreamStub::spawn_with_primary_failure(
            "codex-cli-primary-token",
            "unused-fallback-token",
            "gateway smoke test ok",
            CodexPrimaryFailureMode::Success,
        )
        .await;
        let upstream_url = upstream.base_url.clone();

        with_test_harness_and_codex_upstream(upstream_url, |harness| {
            Box::pin(async move {
                seed_codex_runtime_account(
                    &harness.db.pool,
                    "codex-cli-smoke-account",
                    "codex-cli-primary-token",
                    true,
                )
                .await;

                let (base_url, server) = spawn_router_server(harness.app.clone()).await;
                let home_dir = write_codex_cli_smoke_home(
                    &base_url,
                    &harness.seed.api_key,
                    &harness.seed.allowed_model_slug,
                );

                let output = run_codex_exec_smoke(&home_dir, &harness.seed.allowed_model_slug)
                    .await
                    .expect("codex exec should succeed against gateway responses surface");

                server.abort();
                let _ = fs::remove_dir_all(&home_dir);

                assert!(
                    output.contains("gateway smoke test ok"),
                    "unexpected codex exec output: {output}"
                );
            })
        })
        .await;

        let hits = upstream.recorded_tokens().await;
        assert_eq!(hits, vec!["codex-cli-primary-token".to_string()]);
        upstream.shutdown();
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL with CREATE/DROP DATABASE access"]
    async fn responses_stream_route_still_rejects_models_outside_plan_before_sse() {
        with_test_harness(|harness| {
            Box::pin(async move {
                let (status, body) = send_request(
                    &harness.app,
                    Method::POST,
                    "/v1/responses",
                    Some(json!({
                        "model": harness.seed.blocked_model_slug,
                        "stream": true,
                        "input": "blocked responses stream model"
                    })),
                    Some(&harness.seed.api_key),
                )
                .await;

                assert_eq!(status, StatusCode::FORBIDDEN);
                assert_eq!(body["error"]["message"], "Model not available in your plan");
            })
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL with CREATE/DROP DATABASE access"]
    async fn responses_stream_route_requires_a_valid_api_key_when_keys_are_configured() {
        with_test_harness(|harness| {
            Box::pin(async move {
                let request_body = json!({
                    "model": harness.seed.allowed_model_slug,
                    "stream": true,
                    "input": "auth should fail before responses stream starts"
                });

                let (missing_status, _) = send_request(
                    &harness.app,
                    Method::POST,
                    "/v1/responses",
                    Some(request_body.clone()),
                    None,
                )
                .await;
                assert_eq!(missing_status, StatusCode::UNAUTHORIZED);

                let (invalid_status, _) = send_request(
                    &harness.app,
                    Method::POST,
                    "/v1/responses",
                    Some(request_body),
                    Some("sk-router-invalid"),
                )
                .await;
                assert_eq!(invalid_status, StatusCode::UNAUTHORIZED);
            })
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL with CREATE/DROP DATABASE access"]
    async fn responses_stream_route_ignores_tools_but_rejects_image_payloads_before_sse() {
        with_test_harness(|harness| {
            Box::pin(async move {
                let (tools_status, tools_content_type, tools_body) =
                    send_request_raw_with_content_type(
                        &harness.app,
                        Method::POST,
                        "/v1/responses",
                        Some(json!({
                            "model": harness.seed.allowed_model_slug,
                            "stream": true,
                            "input": "tool payload should fail before responses stream",
                            "tools": [{ "type": "function", "name": "do_work" }]
                        })),
                        Some(&harness.seed.api_key),
                    )
                    .await;
                assert_eq!(tools_status, StatusCode::OK);
                assert_eq!(tools_content_type.as_deref(), Some("text/event-stream"));
                assert!(tools_body.contains("response.failed"));
                assert!(tools_body.contains("No upstream accounts configured"));
                assert!(tools_body.contains("[DONE]"));

                let (image_status, image_body) = send_request(
                    &harness.app,
                    Method::POST,
                    "/v1/responses",
                    Some(json!({
                        "model": harness.seed.allowed_model_slug,
                        "stream": true,
                        "input": {
                            "type": "input_image",
                            "image_url": "https://example.com/test.png"
                        }
                    })),
                    Some(&harness.seed.api_key),
                )
                .await;
                assert_eq!(image_status, StatusCode::BAD_REQUEST);
                assert_eq!(
                    image_body["error"]["message"],
                    "image input is not supported on /v1/responses yet"
                );
            })
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL with CREATE/DROP DATABASE access"]
    async fn chat_completions_route_retries_next_account_after_initial_rate_limit() {
        let upstream = CodexUpstreamStub::spawn(
            "router-primary-token",
            "router-fallback-token",
            "fallback chat response",
        )
        .await;
        let upstream_url = upstream.base_url.clone();

        with_test_harness_and_codex_upstream(upstream_url, |harness| {
            Box::pin(async move {
                let primary_account_id = seed_codex_runtime_account(
                    &harness.db.pool,
                    "codex-router-primary",
                    "router-primary-token",
                    true,
                )
                .await;
                let fallback_account_id = seed_codex_runtime_account(
                    &harness.db.pool,
                    "codex-router-fallback",
                    "router-fallback-token",
                    false,
                )
                .await;

                let (status, body) = send_request(
                    &harness.app,
                    Method::POST,
                    "/v1/chat/completions",
                    Some(json!({
                        "model": harness.seed.allowed_model_slug,
                        "messages": [
                            { "role": "user", "content": "prove route-level failover works" }
                        ]
                    })),
                    Some(&harness.seed.api_key),
                )
                .await;

                assert_eq!(status, StatusCode::OK);
                assert_eq!(
                    body["choices"][0]["message"]["content"],
                    "fallback chat response"
                );

                let primary =
                    crate::db::accounts::get_account(&harness.db.pool, primary_account_id)
                        .await
                        .expect("failed to reload primary account")
                        .expect("primary account should exist");
                assert_eq!(primary.routing_state, ROUTING_STATE_COOLING_DOWN);
                assert_eq!(primary.rate_limit_streak, 1);
                assert_eq!(primary.fail_count, Some(1));

                let fallback =
                    crate::db::accounts::get_account(&harness.db.pool, fallback_account_id)
                        .await
                        .expect("failed to reload fallback account")
                        .expect("fallback account should exist");
                assert_eq!(fallback.routing_state, ROUTING_STATE_HEALTHY);
                assert_eq!(fallback.success_count, Some(1));
                assert_eq!(fallback.fail_count, Some(0));
            })
        })
        .await;

        let hits = upstream.recorded_tokens().await;
        assert_eq!(
            hits,
            vec![
                "router-primary-token".to_string(),
                "router-fallback-token".to_string()
            ]
        );
        upstream.shutdown();
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL with CREATE/DROP DATABASE access"]
    async fn responses_stream_route_retries_next_account_before_emitting_sse() {
        let upstream = CodexUpstreamStub::spawn(
            "responses-primary-token",
            "responses-fallback-token",
            "fallback responses stream",
        )
        .await;
        let upstream_url = upstream.base_url.clone();

        with_test_harness_and_codex_upstream(upstream_url, |harness| {
            Box::pin(async move {
                let primary_account_id = seed_codex_runtime_account(
                    &harness.db.pool,
                    "codex-responses-primary",
                    "responses-primary-token",
                    true,
                )
                .await;
                let fallback_account_id = seed_codex_runtime_account(
                    &harness.db.pool,
                    "codex-responses-fallback",
                    "responses-fallback-token",
                    false,
                )
                .await;

                let (status, content_type, body) = send_request_raw_with_content_type(
                    &harness.app,
                    Method::POST,
                    "/v1/responses",
                    Some(json!({
                        "model": harness.seed.allowed_model_slug,
                        "stream": true,
                        "input": "stream through fallback account"
                    })),
                    Some(&harness.seed.api_key),
                )
                .await;

                assert_eq!(status, StatusCode::OK);
                assert_eq!(content_type.as_deref(), Some("text/event-stream"));
                assert!(body.contains("event: response.output_text.delta"));
                assert!(body.contains("fallback responses stream"));
                assert!(body.contains("event: response.completed"));
                assert!(body.contains("[DONE]"));
                assert!(!body.contains("response.failed"));

                let primary =
                    crate::db::accounts::get_account(&harness.db.pool, primary_account_id)
                        .await
                        .expect("failed to reload primary responses account")
                        .expect("primary responses account should exist");
                assert_eq!(primary.routing_state, ROUTING_STATE_COOLING_DOWN);
                assert_eq!(primary.rate_limit_streak, 1);
                assert_eq!(primary.fail_count, Some(1));

                let fallback =
                    crate::db::accounts::get_account(&harness.db.pool, fallback_account_id)
                        .await
                        .expect("failed to reload fallback responses account")
                        .expect("fallback responses account should exist");
                assert_eq!(fallback.routing_state, ROUTING_STATE_HEALTHY);
                assert_eq!(fallback.success_count, Some(1));
                assert_eq!(fallback.fail_count, Some(0));
            })
        })
        .await;

        let hits = upstream.recorded_tokens().await;
        assert_eq!(
            hits,
            vec![
                "responses-primary-token".to_string(),
                "responses-fallback-token".to_string()
            ]
        );
        upstream.shutdown();
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL with CREATE/DROP DATABASE access"]
    async fn chat_completions_route_retries_next_account_after_initial_unauthorized() {
        let upstream = CodexUpstreamStub::spawn_with_primary_failure(
            "unauthorized-primary-token",
            "unauthorized-fallback-token",
            "fallback after unauthorized",
            CodexPrimaryFailureMode::Unauthorized,
        )
        .await;
        let upstream_url = upstream.base_url.clone();

        with_test_harness_and_codex_upstream(upstream_url, |harness| Box::pin(async move {
            let primary_account_id = seed_codex_runtime_account(
                &harness.db.pool,
                "codex-unauthorized-primary",
                "unauthorized-primary-token",
                true,
            )
            .await;
            let fallback_account_id = seed_codex_runtime_account(
                &harness.db.pool,
                "codex-unauthorized-fallback",
                "unauthorized-fallback-token",
                false,
            )
            .await;

            let (status, body) = send_request(
                &harness.app,
                Method::POST,
                "/v1/chat/completions",
                Some(json!({
                    "model": harness.seed.allowed_model_slug,
                    "messages": [
                        { "role": "user", "content": "unauthorized should fail over before response starts" }
                    ]
                })),
                Some(&harness.seed.api_key),
            )
            .await;

            assert_eq!(status, StatusCode::OK);
            assert_eq!(
                body["choices"][0]["message"]["content"],
                "fallback after unauthorized"
            );

            let primary = crate::db::accounts::get_account(&harness.db.pool, primary_account_id)
                .await
                .expect("failed to reload unauthorized primary account")
                .expect("unauthorized primary account should exist");
            assert_eq!(primary.routing_state, ROUTING_STATE_AUTH_INVALID);
            assert_eq!(
                primary.session_status.as_deref(),
                Some(SESSION_STATUS_EXPIRED)
            );
            assert_eq!(primary.active, Some(false));
            assert_eq!(primary.auth_failure_streak, 1);
            assert_eq!(primary.fail_count, Some(1));

            let fallback = crate::db::accounts::get_account(&harness.db.pool, fallback_account_id)
                .await
                .expect("failed to reload unauthorized fallback account")
                .expect("unauthorized fallback account should exist");
            assert_eq!(fallback.routing_state, ROUTING_STATE_HEALTHY);
            assert_eq!(fallback.success_count, Some(1));
            assert_eq!(fallback.fail_count, Some(0));
        }))
        .await;

        let hits = upstream.recorded_tokens().await;
        assert_eq!(
            hits,
            vec![
                "unauthorized-primary-token".to_string(),
                "unauthorized-fallback-token".to_string()
            ]
        );
        upstream.shutdown();
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL with CREATE/DROP DATABASE access"]
    async fn chat_completions_route_retries_next_account_after_initial_upstream_transient() {
        let upstream = CodexUpstreamStub::spawn_with_primary_failure(
            "transient-primary-token",
            "transient-fallback-token",
            "fallback after upstream transient",
            CodexPrimaryFailureMode::UpstreamTransient,
        )
        .await;
        let upstream_url = upstream.base_url.clone();

        with_test_harness_and_codex_upstream(upstream_url, |harness| Box::pin(async move {
            let primary_account_id = seed_codex_runtime_account(
                &harness.db.pool,
                "codex-transient-primary",
                "transient-primary-token",
                true,
            )
            .await;
            let fallback_account_id = seed_codex_runtime_account(
                &harness.db.pool,
                "codex-transient-fallback",
                "transient-fallback-token",
                false,
            )
            .await;

            let (status, body) = send_request(
                &harness.app,
                Method::POST,
                "/v1/chat/completions",
                Some(json!({
                    "model": harness.seed.allowed_model_slug,
                    "messages": [
                        { "role": "user", "content": "transient upstream should fail over before response starts" }
                    ]
                })),
                Some(&harness.seed.api_key),
            )
            .await;

            assert_eq!(status, StatusCode::OK);
            assert_eq!(
                body["choices"][0]["message"]["content"],
                "fallback after upstream transient"
            );

            let primary = crate::db::accounts::get_account(&harness.db.pool, primary_account_id)
                .await
                .expect("failed to reload transient primary account")
                .expect("transient primary account should exist");
            assert_eq!(primary.routing_state, ROUTING_STATE_COOLING_DOWN);
            assert_eq!(primary.rate_limit_streak, 0);
            assert_eq!(primary.fail_count, Some(1));
            assert_eq!(primary.active, Some(true));
            assert!(
                primary
                    .last_routing_error
                    .as_deref()
                    .unwrap_or_default()
                    .contains("500")
            );

            let fallback = crate::db::accounts::get_account(&harness.db.pool, fallback_account_id)
                .await
                .expect("failed to reload transient fallback account")
                .expect("transient fallback account should exist");
            assert_eq!(fallback.routing_state, ROUTING_STATE_HEALTHY);
            assert_eq!(fallback.success_count, Some(1));
            assert_eq!(fallback.fail_count, Some(0));
        }))
        .await;

        let hits = upstream.recorded_tokens().await;
        assert_eq!(
            hits,
            vec![
                "transient-primary-token".to_string(),
                "transient-fallback-token".to_string()
            ]
        );
        upstream.shutdown();
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL with CREATE/DROP DATABASE access"]
    async fn chat_completions_route_deactivates_failed_proxy_before_retrying() {
        let proxy_hits = Arc::new(RwLock::new(Vec::new()));
        let failed_proxy = HttpProxyStub::spawn(
            "failed-proxy",
            ProxyStubMode::CfBlocked,
            None,
            proxy_hits.clone(),
        )
        .await;
        let replacement_proxy = HttpProxyStub::spawn(
            "replacement-proxy",
            ProxyStubMode::Success,
            Some("fallback through replacement proxy"),
            proxy_hits,
        )
        .await;
        let failed_proxy_url = failed_proxy.url.clone();
        let replacement_proxy_url = replacement_proxy.url.clone();

        with_test_harness_and_codex_upstream(
            "http://codex.invalid".to_string(),
            |harness| {
                Box::pin(async move {
                    let account_id = seed_codex_runtime_account(
                        &harness.db.pool,
                        "codex-proxy-retry-account",
                        "proxy-route-token",
                        true,
                    )
                    .await;
                    let peer_account_id = seed_codex_runtime_account(
                        &harness.db.pool,
                        "codex-proxy-retry-peer-account",
                        "proxy-route-peer-token",
                        false,
                    )
                    .await;
                    let failed_proxy_id =
                        seed_proxy(&harness.db.pool, &failed_proxy_url, "failed-proxy").await;
                    let replacement_proxy_id = seed_proxy(
                        &harness.db.pool,
                        &replacement_proxy_url,
                        "replacement-proxy",
                    )
                    .await;
                    crate::db::accounts::assign_proxy_to_account(
                        &harness.db.pool,
                        account_id,
                        Some(failed_proxy_id),
                    )
                    .await
                    .expect("failed to assign failing proxy to test account");
                    crate::db::accounts::assign_proxy_to_account(
                        &harness.db.pool,
                        peer_account_id,
                        Some(failed_proxy_id),
                    )
                    .await
                    .expect("failed to assign failing proxy to peer account");

                    let (status, body) = send_request(
                        &harness.app,
                        Method::POST,
                        "/v1/chat/completions",
                        Some(json!({
                            "model": harness.seed.allowed_model_slug,
                            "messages": [
                                { "role": "user", "content": "proxy failover should succeed before response starts" }
                            ]
                        })),
                        Some(&harness.seed.api_key),
                    )
                    .await;

                    assert_eq!(status, StatusCode::OK);
                    assert_eq!(
                        body["choices"][0]["message"]["content"],
                        "fallback through replacement proxy"
                    );

                    let failed = crate::db::proxies::get_proxy(&harness.db.pool, failed_proxy_id)
                        .await
                        .expect("failed to reload failed proxy")
                        .expect("failed proxy should exist");
                    assert_eq!(failed.active, Some(false));

                    let replacement = crate::db::proxies::get_proxy(
                        &harness.db.pool,
                        replacement_proxy_id,
                    )
                    .await
                    .expect("failed to reload replacement proxy")
                    .expect("replacement proxy should exist");
                    assert_eq!(replacement.active, Some(true));

                    let account = crate::db::accounts::get_account(&harness.db.pool, account_id)
                        .await
                        .expect("failed to reload proxy retry account")
                        .expect("proxy retry account should exist");
                    assert_eq!(account.proxy_id, Some(replacement_proxy_id));
                    assert_eq!(account.success_count, Some(1));

                    let peer_account =
                        crate::db::accounts::get_account(&harness.db.pool, peer_account_id)
                            .await
                            .expect("failed to reload proxy retry peer account")
                            .expect("proxy retry peer account should exist");
                    assert_eq!(peer_account.proxy_id, Some(replacement_proxy_id));
                })
            },
        )
        .await;

        let hits = replacement_proxy.recorded_hits().await;
        assert_eq!(
            hits,
            vec!["failed-proxy".to_string(), "replacement-proxy".to_string()]
        );
        failed_proxy.shutdown();
        replacement_proxy.shutdown();
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL with CREATE/DROP DATABASE access"]
    async fn chat_completions_route_deactivates_unreachable_proxy_before_retrying() {
        let proxy_hits = Arc::new(RwLock::new(Vec::new()));
        let replacement_proxy = HttpProxyStub::spawn(
            "replacement-proxy",
            ProxyStubMode::Success,
            Some("fallback after unreachable proxy"),
            proxy_hits,
        )
        .await;
        let unreachable_proxy_url = "http://127.0.0.1:9".to_string();
        let replacement_proxy_url = replacement_proxy.url.clone();

        with_test_harness_and_codex_upstream(
            "http://codex.invalid".to_string(),
            |harness| {
                Box::pin(async move {
                    let account_id = seed_codex_runtime_account(
                        &harness.db.pool,
                        "codex-unreachable-proxy-account",
                        "proxy-transport-token",
                        true,
                    )
                    .await;
                    let failed_proxy_id =
                        seed_proxy(&harness.db.pool, &unreachable_proxy_url, "dead-proxy").await;
                    let replacement_proxy_id = seed_proxy(
                        &harness.db.pool,
                        &replacement_proxy_url,
                        "replacement-proxy",
                    )
                    .await;
                    crate::db::accounts::assign_proxy_to_account(
                        &harness.db.pool,
                        account_id,
                        Some(failed_proxy_id),
                    )
                    .await
                    .expect("failed to assign unreachable proxy to test account");

                    let (status, body) = send_request(
                        &harness.app,
                        Method::POST,
                        "/v1/chat/completions",
                        Some(json!({
                            "model": harness.seed.allowed_model_slug,
                            "messages": [
                                { "role": "user", "content": "dead proxy should fail over before response starts" }
                            ]
                        })),
                        Some(&harness.seed.api_key),
                    )
                    .await;

                    assert_eq!(status, StatusCode::OK);
                    assert_eq!(
                        body["choices"][0]["message"]["content"],
                        "fallback after unreachable proxy"
                    );

                    let failed = crate::db::proxies::get_proxy(&harness.db.pool, failed_proxy_id)
                        .await
                        .expect("failed to reload dead proxy")
                        .expect("dead proxy should exist");
                    assert_eq!(failed.active, Some(false));

                    let account = crate::db::accounts::get_account(&harness.db.pool, account_id)
                        .await
                        .expect("failed to reload unreachable proxy account")
                        .expect("unreachable proxy account should exist");
                    assert_eq!(account.proxy_id, Some(replacement_proxy_id));
                    assert_eq!(account.success_count, Some(1));
                })
            },
        )
        .await;

        let hits = replacement_proxy.recorded_hits().await;
        assert_eq!(hits, vec!["replacement-proxy".to_string()]);
        replacement_proxy.shutdown();
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL with CREATE/DROP DATABASE access"]
    async fn chat_completions_stream_route_fails_closed_and_expires_account_after_post_stream_unauthorized()
     {
        let upstream = CodexUpstreamStub::spawn_with_primary_failure(
            "post-stream-unauthorized-token",
            "unused-fallback-token",
            "unused fallback",
            CodexPrimaryFailureMode::PostStreamUnauthorized,
        )
        .await;
        let upstream_url = upstream.base_url.clone();

        with_test_harness_and_codex_upstream(upstream_url, |harness| {
            Box::pin(async move {
                let account_id = seed_codex_runtime_account(
                    &harness.db.pool,
                    "codex-post-stream-unauthorized-account",
                    "post-stream-unauthorized-token",
                    true,
                )
                .await;

                let (status, body) = send_request_raw(
                    &harness.app,
                    Method::POST,
                    "/v1/chat/completions",
                    Some(json!({
                        "model": harness.seed.allowed_model_slug,
                        "stream": true,
                        "messages": [
                            { "role": "user", "content": "post-stream unauthorized should fail closed" }
                        ]
                    })),
                    Some(&harness.seed.api_key),
                )
                .await;

                assert_eq!(status, StatusCode::OK);
                assert!(body.contains(r#""role":"assistant""#), "body={body}");
                assert!(body.contains("partial token before auth failure"));
                assert!(body.contains(r#""message":"Unauthorized""#));
                assert_substrings_in_order(
                    &body,
                    &[
                        r#""role":"assistant""#,
                        "partial token before auth failure",
                        r#""message":"Unauthorized""#,
                    ],
                );
                assert!(!body.contains("[DONE]"));
                assert!(!body.contains(r#""finish_reason":"stop""#));

                let account = crate::db::accounts::get_account(&harness.db.pool, account_id)
                    .await
                    .expect("failed to reload post-stream unauthorized account")
                    .expect("post-stream unauthorized account should exist");
                assert_eq!(account.routing_state, ROUTING_STATE_AUTH_INVALID);
                assert_eq!(
                    account.session_status.as_deref(),
                    Some(SESSION_STATUS_EXPIRED)
                );
                assert_eq!(account.active, Some(false));
                assert_eq!(account.auth_failure_streak, 1);
                assert_eq!(account.fail_count, Some(1));
            })
        })
        .await;

        let hits = upstream.recorded_tokens().await;
        assert_eq!(hits, vec!["post-stream-unauthorized-token".to_string()]);
        upstream.shutdown();
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL with CREATE/DROP DATABASE access"]
    async fn chat_completions_stream_route_fails_closed_and_deactivates_proxy_after_post_stream_cf_blocked()
     {
        let proxy_hits = Arc::new(RwLock::new(Vec::new()));
        let failed_proxy = HttpProxyStub::spawn(
            "failed-proxy",
            ProxyStubMode::PostStreamCfBlocked,
            None,
            proxy_hits.clone(),
        )
        .await;
        let replacement_proxy = HttpProxyStub::spawn(
            "replacement-proxy",
            ProxyStubMode::Success,
            Some("unused replacement proxy response"),
            proxy_hits,
        )
        .await;
        let failed_proxy_url = failed_proxy.url.clone();
        let replacement_proxy_url = replacement_proxy.url.clone();

        with_test_harness_and_codex_upstream("http://codex.invalid".to_string(), |harness| {
            Box::pin(async move {
                let account_id = seed_codex_runtime_account(
                    &harness.db.pool,
                    "codex-chat-post-stream-proxy-account",
                    "chat-post-stream-proxy-token",
                    true,
                )
                .await;
                let peer_account_id = seed_codex_runtime_account(
                    &harness.db.pool,
                    "codex-chat-post-stream-proxy-peer-account",
                    "chat-post-stream-proxy-peer-token",
                    false,
                )
                .await;
                let failed_proxy_id =
                    seed_proxy(&harness.db.pool, &failed_proxy_url, "failed-proxy").await;
                let replacement_proxy_id = seed_proxy(
                    &harness.db.pool,
                    &replacement_proxy_url,
                    "replacement-proxy",
                )
                .await;
                crate::db::accounts::assign_proxy_to_account(
                    &harness.db.pool,
                    account_id,
                    Some(failed_proxy_id),
                )
                .await
                .expect("failed to assign failing chat post-stream proxy to account");
                crate::db::accounts::assign_proxy_to_account(
                    &harness.db.pool,
                    peer_account_id,
                    Some(failed_proxy_id),
                )
                .await
                .expect("failed to assign failing chat post-stream proxy to peer account");

                let (status, body) = send_request_raw(
                    &harness.app,
                    Method::POST,
                    "/v1/chat/completions",
                    Some(json!({
                        "model": harness.seed.allowed_model_slug,
                        "stream": true,
                        "messages": [
                            { "role": "user", "content": "post-stream proxy failure should fail closed" }
                        ]
                    })),
                    Some(&harness.seed.api_key),
                )
                .await;

                assert_eq!(status, StatusCode::OK);
                assert!(body.contains(r#""role":"assistant""#));
                assert!(body.contains("partial token before proxy failure"));
                assert!(body.contains(r#""message":"Cloudflare blocked""#));
                assert_substrings_in_order(
                    &body,
                    &[
                        r#""role":"assistant""#,
                        "partial token before proxy failure",
                        r#""message":"Cloudflare blocked""#,
                    ],
                );
                assert!(!body.contains("[DONE]"));
                assert!(!body.contains(r#""finish_reason":"stop""#));

                let failed = crate::db::proxies::get_proxy(&harness.db.pool, failed_proxy_id)
                    .await
                    .expect("failed to reload post-stream failed chat proxy")
                    .expect("post-stream failed chat proxy should exist");
                assert_eq!(failed.active, Some(false));

                let account = crate::db::accounts::get_account(&harness.db.pool, account_id)
                    .await
                    .expect("failed to reload chat post-stream proxy account")
                    .expect("chat post-stream proxy account should exist");
                assert_eq!(account.proxy_id, Some(replacement_proxy_id));
                assert_eq!(account.routing_state, ROUTING_STATE_COOLING_DOWN);
                assert_eq!(account.fail_count, Some(1));
                assert_eq!(
                    account.last_routing_error.as_deref(),
                    Some("Cloudflare blocked")
                );

                let peer_account =
                    crate::db::accounts::get_account(&harness.db.pool, peer_account_id)
                        .await
                        .expect("failed to reload chat post-stream proxy peer account")
                        .expect("chat post-stream proxy peer account should exist");
                assert_eq!(peer_account.proxy_id, Some(replacement_proxy_id));
            })
        })
        .await;

        let hits = replacement_proxy.recorded_hits().await;
        assert_eq!(hits, vec!["failed-proxy".to_string()]);
        failed_proxy.shutdown();
        replacement_proxy.shutdown();
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL with CREATE/DROP DATABASE access"]
    async fn chat_completions_stream_route_reports_unexpected_stream_end_after_partial_output() {
        let upstream = CodexUpstreamStub::spawn_with_primary_failure(
            "post-stream-truncated-token",
            "unused-fallback-token",
            "unused fallback",
            CodexPrimaryFailureMode::PostStreamTruncated,
        )
        .await;
        let upstream_url = upstream.base_url.clone();

        with_test_harness_and_codex_upstream(upstream_url, |harness| {
            Box::pin(async move {
                let account_id = seed_codex_runtime_account(
                    &harness.db.pool,
                    "codex-post-stream-truncated-account",
                    "post-stream-truncated-token",
                    true,
                )
                .await;

                let (status, body) = send_request_raw(
                    &harness.app,
                    Method::POST,
                    "/v1/chat/completions",
                    Some(json!({
                        "model": harness.seed.allowed_model_slug,
                        "stream": true,
                        "messages": [
                            { "role": "user", "content": "truncated post-stream chat should fail closed" }
                        ]
                    })),
                    Some(&harness.seed.api_key),
                )
                .await;

                assert_eq!(status, StatusCode::OK);
                assert!(body.contains(r#""role":"assistant""#));
                assert!(body.contains("partial token before truncated stream"));
                assert!(body.contains(r#""message":"Upstream stream ended unexpectedly""#));
                assert_substrings_in_order(
                    &body,
                    &[
                        r#""role":"assistant""#,
                        "partial token before truncated stream",
                        r#""message":"Upstream stream ended unexpectedly""#,
                    ],
                );
                assert!(!body.contains("[DONE]"));
                assert!(!body.contains(r#""finish_reason":"stop""#));

                let account = crate::db::accounts::get_account(&harness.db.pool, account_id)
                    .await
                    .expect("failed to reload post-stream truncated account")
                    .expect("post-stream truncated account should exist");
                assert_eq!(account.routing_state, ROUTING_STATE_COOLING_DOWN);
                assert_eq!(account.fail_count, Some(1));
                assert_eq!(
                    account.last_routing_error.as_deref(),
                    Some("Generic upstream failure pushed this account into cooldown.")
                );
            })
        })
        .await;

        let hits = upstream.recorded_tokens().await;
        assert_eq!(hits, vec!["post-stream-truncated-token".to_string()]);
        upstream.shutdown();
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL with CREATE/DROP DATABASE access"]
    async fn chat_completions_stream_route_reports_parser_error_after_partial_output() {
        let upstream = CodexUpstreamStub::spawn_with_primary_failure(
            "post-stream-invalid-json-token",
            "unused-fallback-token",
            "unused fallback",
            CodexPrimaryFailureMode::PostStreamInvalidJson,
        )
        .await;
        let upstream_url = upstream.base_url.clone();

        with_test_harness_and_codex_upstream(upstream_url, |harness| {
            Box::pin(async move {
                let account_id = seed_codex_runtime_account(
                    &harness.db.pool,
                    "codex-post-stream-invalid-json-account",
                    "post-stream-invalid-json-token",
                    true,
                )
                .await;

                let (status, body) = send_request_raw(
                    &harness.app,
                    Method::POST,
                    "/v1/chat/completions",
                    Some(json!({
                        "model": harness.seed.allowed_model_slug,
                        "stream": true,
                        "messages": [
                            { "role": "user", "content": "invalid sse json after output should fail closed" }
                        ]
                    })),
                    Some(&harness.seed.api_key),
                )
                .await;

                assert_eq!(status, StatusCode::OK);
                assert!(body.contains(r#""role":"assistant""#));
                assert!(body.contains("partial token before parser failure"));
                assert!(body.contains(r#""message":"Invalid Codex SSE payload for event Some(\"response.failed\")"#));
                assert_substrings_in_order(
                    &body,
                    &[
                        r#""role":"assistant""#,
                        "partial token before parser failure",
                        r#""message":"Invalid Codex SSE payload for event Some(\"response.failed\")"#,
                    ],
                );
                assert!(!body.contains("[DONE]"));
                assert!(!body.contains(r#""finish_reason":"stop""#));

                let account = crate::db::accounts::get_account(&harness.db.pool, account_id)
                    .await
                    .expect("failed to reload post-stream invalid json account")
                    .expect("post-stream invalid json account should exist");
                assert_eq!(account.routing_state, ROUTING_STATE_COOLING_DOWN);
                assert_eq!(account.fail_count, Some(1));
                assert!(
                    account
                        .last_routing_error
                        .as_deref()
                        .unwrap_or_default()
                        .contains("Invalid Codex SSE payload for event Some(\"response.failed\")")
                );
            })
        })
        .await;

        let hits = upstream.recorded_tokens().await;
        assert_eq!(hits, vec!["post-stream-invalid-json-token".to_string()]);
        upstream.shutdown();
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL with CREATE/DROP DATABASE access"]
    async fn responses_stream_route_retries_next_account_after_initial_unauthorized() {
        let upstream = CodexUpstreamStub::spawn_with_primary_failure(
            "responses-unauthorized-primary-token",
            "responses-unauthorized-fallback-token",
            "fallback responses after unauthorized",
            CodexPrimaryFailureMode::Unauthorized,
        )
        .await;
        let upstream_url = upstream.base_url.clone();

        with_test_harness_and_codex_upstream(upstream_url, |harness| {
            Box::pin(async move {
                let primary_account_id = seed_codex_runtime_account(
                    &harness.db.pool,
                    "codex-responses-unauthorized-primary",
                    "responses-unauthorized-primary-token",
                    true,
                )
                .await;
                let fallback_account_id = seed_codex_runtime_account(
                    &harness.db.pool,
                    "codex-responses-unauthorized-fallback",
                    "responses-unauthorized-fallback-token",
                    false,
                )
                .await;

                let (status, content_type, body) = send_request_raw_with_content_type(
                    &harness.app,
                    Method::POST,
                    "/v1/responses",
                    Some(json!({
                        "model": harness.seed.allowed_model_slug,
                        "stream": true,
                        "input": "unauthorized responses should fail over before SSE starts"
                    })),
                    Some(&harness.seed.api_key),
                )
                .await;

                assert_eq!(status, StatusCode::OK);
                assert_eq!(content_type.as_deref(), Some("text/event-stream"));
                assert!(
                    body.contains("event: response.output_text.delta"),
                    "body={body}"
                );
                assert!(body.contains("fallback responses after unauthorized"));
                assert!(body.contains("event: response.completed"));
                assert!(body.contains("[DONE]"));
                assert!(!body.contains("response.failed"));

                let primary =
                    crate::db::accounts::get_account(&harness.db.pool, primary_account_id)
                        .await
                        .expect("failed to reload unauthorized primary responses account")
                        .expect("unauthorized primary responses account should exist");
                assert_eq!(primary.routing_state, ROUTING_STATE_AUTH_INVALID);
                assert_eq!(
                    primary.session_status.as_deref(),
                    Some(SESSION_STATUS_EXPIRED)
                );
                assert_eq!(primary.active, Some(false));
                assert_eq!(primary.auth_failure_streak, 1);
                assert_eq!(primary.fail_count, Some(1));

                let fallback =
                    crate::db::accounts::get_account(&harness.db.pool, fallback_account_id)
                        .await
                        .expect("failed to reload unauthorized fallback responses account")
                        .expect("unauthorized fallback responses account should exist");
                assert_eq!(fallback.routing_state, ROUTING_STATE_HEALTHY);
                assert_eq!(fallback.success_count, Some(1));
                assert_eq!(fallback.fail_count, Some(0));
            })
        })
        .await;

        let hits = upstream.recorded_tokens().await;
        assert_eq!(
            hits,
            vec![
                "responses-unauthorized-primary-token".to_string(),
                "responses-unauthorized-fallback-token".to_string()
            ]
        );
        upstream.shutdown();
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL with CREATE/DROP DATABASE access"]
    async fn responses_stream_route_deactivates_failed_proxy_before_emitting_sse() {
        let proxy_hits = Arc::new(RwLock::new(Vec::new()));
        let failed_proxy = HttpProxyStub::spawn(
            "failed-proxy",
            ProxyStubMode::CfBlocked,
            None,
            proxy_hits.clone(),
        )
        .await;
        let replacement_proxy = HttpProxyStub::spawn(
            "replacement-proxy",
            ProxyStubMode::Success,
            Some("responses via replacement proxy"),
            proxy_hits,
        )
        .await;
        let failed_proxy_url = failed_proxy.url.clone();
        let replacement_proxy_url = replacement_proxy.url.clone();

        with_test_harness_and_codex_upstream("http://codex.invalid".to_string(), |harness| {
            Box::pin(async move {
                let account_id = seed_codex_runtime_account(
                    &harness.db.pool,
                    "codex-responses-proxy-retry-account",
                    "responses-proxy-route-token",
                    true,
                )
                .await;
                let failed_proxy_id =
                    seed_proxy(&harness.db.pool, &failed_proxy_url, "failed-proxy").await;
                let replacement_proxy_id = seed_proxy(
                    &harness.db.pool,
                    &replacement_proxy_url,
                    "replacement-proxy",
                )
                .await;
                crate::db::accounts::assign_proxy_to_account(
                    &harness.db.pool,
                    account_id,
                    Some(failed_proxy_id),
                )
                .await
                .expect("failed to assign failing responses proxy to test account");

                let (status, content_type, body) = send_request_raw_with_content_type(
                    &harness.app,
                    Method::POST,
                    "/v1/responses",
                    Some(json!({
                        "model": harness.seed.allowed_model_slug,
                        "stream": true,
                        "input": "proxy failover should succeed before first SSE event"
                    })),
                    Some(&harness.seed.api_key),
                )
                .await;

                assert_eq!(status, StatusCode::OK);
                assert_eq!(content_type.as_deref(), Some("text/event-stream"));
                assert!(body.contains("event: response.output_text.delta"));
                assert!(body.contains("responses via replacement proxy"));
                assert!(body.contains("event: response.completed"));
                assert!(body.contains("[DONE]"));
                assert!(!body.contains("response.failed"));

                let failed = crate::db::proxies::get_proxy(&harness.db.pool, failed_proxy_id)
                    .await
                    .expect("failed to reload failed responses proxy")
                    .expect("failed responses proxy should exist");
                assert_eq!(failed.active, Some(false));

                let replacement =
                    crate::db::proxies::get_proxy(&harness.db.pool, replacement_proxy_id)
                        .await
                        .expect("failed to reload replacement responses proxy")
                        .expect("replacement responses proxy should exist");
                assert_eq!(replacement.active, Some(true));

                let account = crate::db::accounts::get_account(&harness.db.pool, account_id)
                    .await
                    .expect("failed to reload responses proxy retry account")
                    .expect("responses proxy retry account should exist");
                assert_eq!(account.proxy_id, Some(replacement_proxy_id));
                assert_eq!(account.success_count, Some(1));
            })
        })
        .await;

        let hits = replacement_proxy.recorded_hits().await;
        assert_eq!(
            hits,
            vec!["failed-proxy".to_string(), "replacement-proxy".to_string()]
        );
        failed_proxy.shutdown();
        replacement_proxy.shutdown();
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL with CREATE/DROP DATABASE access"]
    async fn responses_stream_route_reports_unexpected_stream_end_after_partial_output() {
        let upstream = CodexUpstreamStub::spawn_with_primary_failure(
            "responses-post-stream-truncated-token",
            "unused-fallback-token",
            "unused fallback",
            CodexPrimaryFailureMode::PostStreamTruncated,
        )
        .await;
        let upstream_url = upstream.base_url.clone();

        with_test_harness_and_codex_upstream(upstream_url, |harness| {
            Box::pin(async move {
                let account_id = seed_codex_runtime_account(
                    &harness.db.pool,
                    "codex-responses-post-stream-truncated-account",
                    "responses-post-stream-truncated-token",
                    true,
                )
                .await;

                let (status, content_type, body) = send_request_raw_with_content_type(
                    &harness.app,
                    Method::POST,
                    "/v1/responses",
                    Some(json!({
                        "model": harness.seed.allowed_model_slug,
                        "stream": true,
                        "input": "truncated post-stream responses should fail closed"
                    })),
                    Some(&harness.seed.api_key),
                )
                .await;

                assert_eq!(status, StatusCode::OK);
                assert_eq!(content_type.as_deref(), Some("text/event-stream"));
                assert!(body.contains("event: response.output_text.delta"));
                assert!(body.contains("partial token before truncated stream"));
                assert!(body.contains("event: response.failed"));
                assert!(body.contains(r#""message":"Upstream stream ended unexpectedly""#));
                assert_substrings_in_order(
                    &body,
                    &[
                        "event: response.output_text.delta",
                        "partial token before truncated stream",
                        "event: response.failed",
                        r#""message":"Upstream stream ended unexpectedly""#,
                        "[DONE]",
                    ],
                );
                assert!(body.contains("[DONE]"));
                assert!(!body.contains("event: response.completed"));

                let account = crate::db::accounts::get_account(&harness.db.pool, account_id)
                    .await
                    .expect("failed to reload responses post-stream truncated account")
                    .expect("responses post-stream truncated account should exist");
                assert_eq!(account.routing_state, ROUTING_STATE_COOLING_DOWN);
                assert_eq!(account.fail_count, Some(1));
                assert_eq!(
                    account.last_routing_error.as_deref(),
                    Some("Generic upstream failure pushed this account into cooldown.")
                );
            })
        })
        .await;

        let hits = upstream.recorded_tokens().await;
        assert_eq!(
            hits,
            vec!["responses-post-stream-truncated-token".to_string()]
        );
        upstream.shutdown();
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL with CREATE/DROP DATABASE access"]
    async fn responses_stream_route_reports_parser_error_after_partial_output() {
        let upstream = CodexUpstreamStub::spawn_with_primary_failure(
            "responses-post-stream-invalid-json-token",
            "unused-fallback-token",
            "unused fallback",
            CodexPrimaryFailureMode::PostStreamInvalidJson,
        )
        .await;
        let upstream_url = upstream.base_url.clone();

        with_test_harness_and_codex_upstream(upstream_url, |harness| {
            Box::pin(async move {
                let account_id = seed_codex_runtime_account(
                    &harness.db.pool,
                    "codex-responses-post-stream-invalid-json-account",
                    "responses-post-stream-invalid-json-token",
                    true,
                )
                .await;

                let (status, content_type, body) = send_request_raw_with_content_type(
                    &harness.app,
                    Method::POST,
                    "/v1/responses",
                    Some(json!({
                        "model": harness.seed.allowed_model_slug,
                        "stream": true,
                        "input": "invalid sse json after output should fail closed on responses"
                    })),
                    Some(&harness.seed.api_key),
                )
                .await;

                assert_eq!(status, StatusCode::OK);
                assert_eq!(content_type.as_deref(), Some("text/event-stream"));
                assert!(body.contains("event: response.output_text.delta"));
                assert!(body.contains("partial token before parser failure"));
                assert!(body.contains("event: response.failed"));
                assert!(body.contains(r#""message":"Invalid Codex SSE payload for event Some(\"response.failed\")"#));
                assert_substrings_in_order(
                    &body,
                    &[
                        "event: response.output_text.delta",
                        "partial token before parser failure",
                        "event: response.failed",
                        r#""message":"Invalid Codex SSE payload for event Some(\"response.failed\")"#,
                        "[DONE]",
                    ],
                );
                assert!(body.contains("[DONE]"));
                assert!(!body.contains("event: response.completed"));

                let account = crate::db::accounts::get_account(&harness.db.pool, account_id)
                    .await
                    .expect("failed to reload responses post-stream invalid json account")
                    .expect("responses post-stream invalid json account should exist");
                assert_eq!(account.routing_state, ROUTING_STATE_COOLING_DOWN);
                assert_eq!(account.fail_count, Some(1));
                assert!(
                    account
                        .last_routing_error
                        .as_deref()
                        .unwrap_or_default()
                        .contains("Invalid Codex SSE payload for event Some(\"response.failed\")")
                );
            })
        })
        .await;

        let hits = upstream.recorded_tokens().await;
        assert_eq!(
            hits,
            vec!["responses-post-stream-invalid-json-token".to_string()]
        );
        upstream.shutdown();
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL with CREATE/DROP DATABASE access"]
    async fn responses_stream_route_fails_closed_and_expires_account_after_post_stream_unauthorized()
     {
        let upstream = CodexUpstreamStub::spawn_with_primary_failure(
            "responses-post-stream-unauthorized-token",
            "unused-fallback-token",
            "unused fallback",
            CodexPrimaryFailureMode::PostStreamUnauthorized,
        )
        .await;
        let upstream_url = upstream.base_url.clone();

        with_test_harness_and_codex_upstream(upstream_url, |harness| {
            Box::pin(async move {
                let account_id = seed_codex_runtime_account(
                    &harness.db.pool,
                    "codex-responses-post-stream-unauthorized-account",
                    "responses-post-stream-unauthorized-token",
                    true,
                )
                .await;

                let (status, content_type, body) = send_request_raw_with_content_type(
                    &harness.app,
                    Method::POST,
                    "/v1/responses",
                    Some(json!({
                        "model": harness.seed.allowed_model_slug,
                        "stream": true,
                        "input": "post-stream unauthorized should fail closed on responses"
                    })),
                    Some(&harness.seed.api_key),
                )
                .await;

                assert_eq!(status, StatusCode::OK);
                assert_eq!(content_type.as_deref(), Some("text/event-stream"));
                assert!(body.contains("event: response.output_text.delta"));
                assert!(body.contains("partial token before auth failure"));
                assert!(body.contains("event: response.failed"));
                assert!(body.contains(r#""message":"Unauthorized""#));
                assert_substrings_in_order(
                    &body,
                    &[
                        "event: response.output_text.delta",
                        "partial token before auth failure",
                        "event: response.failed",
                        r#""message":"Unauthorized""#,
                        "[DONE]",
                    ],
                );
                assert!(body.contains("[DONE]"));
                assert!(!body.contains("event: response.completed"));

                let account = crate::db::accounts::get_account(&harness.db.pool, account_id)
                    .await
                    .expect("failed to reload responses post-stream unauthorized account")
                    .expect("responses post-stream unauthorized account should exist");
                assert_eq!(account.routing_state, ROUTING_STATE_AUTH_INVALID);
                assert_eq!(
                    account.session_status.as_deref(),
                    Some(SESSION_STATUS_EXPIRED)
                );
                assert_eq!(account.active, Some(false));
                assert_eq!(account.auth_failure_streak, 1);
                assert_eq!(account.fail_count, Some(1));
            })
        })
        .await;

        let hits = upstream.recorded_tokens().await;
        assert_eq!(
            hits,
            vec!["responses-post-stream-unauthorized-token".to_string()]
        );
        upstream.shutdown();
    }

    #[tokio::test]
    #[ignore = "requires DATABASE_URL with CREATE/DROP DATABASE access"]
    async fn responses_stream_route_fails_closed_and_deactivates_proxy_after_post_stream_cf_blocked()
     {
        let proxy_hits = Arc::new(RwLock::new(Vec::new()));
        let failed_proxy = HttpProxyStub::spawn(
            "failed-proxy",
            ProxyStubMode::PostStreamCfBlocked,
            None,
            proxy_hits.clone(),
        )
        .await;
        let replacement_proxy = HttpProxyStub::spawn(
            "replacement-proxy",
            ProxyStubMode::Success,
            Some("unused replacement proxy response"),
            proxy_hits,
        )
        .await;
        let failed_proxy_url = failed_proxy.url.clone();
        let replacement_proxy_url = replacement_proxy.url.clone();

        with_test_harness_and_codex_upstream("http://codex.invalid".to_string(), |harness| {
            Box::pin(async move {
                let account_id = seed_codex_runtime_account(
                    &harness.db.pool,
                    "codex-post-stream-proxy-account",
                    "post-stream-proxy-token",
                    true,
                )
                .await;
                let peer_account_id = seed_codex_runtime_account(
                    &harness.db.pool,
                    "codex-post-stream-proxy-peer-account",
                    "post-stream-proxy-peer-token",
                    false,
                )
                .await;
                let failed_proxy_id =
                    seed_proxy(&harness.db.pool, &failed_proxy_url, "failed-proxy").await;
                let replacement_proxy_id = seed_proxy(
                    &harness.db.pool,
                    &replacement_proxy_url,
                    "replacement-proxy",
                )
                .await;
                crate::db::accounts::assign_proxy_to_account(
                    &harness.db.pool,
                    account_id,
                    Some(failed_proxy_id),
                )
                .await
                .expect("failed to assign failing post-stream proxy to account");
                crate::db::accounts::assign_proxy_to_account(
                    &harness.db.pool,
                    peer_account_id,
                    Some(failed_proxy_id),
                )
                .await
                .expect("failed to assign failing post-stream proxy to peer account");

                let (status, content_type, body) = send_request_raw_with_content_type(
                    &harness.app,
                    Method::POST,
                    "/v1/responses",
                    Some(json!({
                        "model": harness.seed.allowed_model_slug,
                        "stream": true,
                        "input": "post-stream cf blocked should fail closed"
                    })),
                    Some(&harness.seed.api_key),
                )
                .await;

                assert_eq!(status, StatusCode::OK);
                assert_eq!(content_type.as_deref(), Some("text/event-stream"));
                assert!(body.contains("event: response.output_text.delta"));
                assert!(body.contains("partial token before proxy failure"));
                assert!(body.contains("event: response.failed"));
                assert!(body.contains(r#""message":"Cloudflare blocked""#));
                assert_substrings_in_order(
                    &body,
                    &[
                        "event: response.output_text.delta",
                        "partial token before proxy failure",
                        "event: response.failed",
                        r#""message":"Cloudflare blocked""#,
                        "[DONE]",
                    ],
                );
                assert!(body.contains("[DONE]"));
                assert!(!body.contains("event: response.completed"));

                let failed = crate::db::proxies::get_proxy(&harness.db.pool, failed_proxy_id)
                    .await
                    .expect("failed to reload post-stream failed proxy")
                    .expect("post-stream failed proxy should exist");
                assert_eq!(failed.active, Some(false));

                let account = crate::db::accounts::get_account(&harness.db.pool, account_id)
                    .await
                    .expect("failed to reload post-stream proxy account")
                    .expect("post-stream proxy account should exist");
                assert_eq!(account.proxy_id, Some(replacement_proxy_id));
                assert_eq!(account.routing_state, ROUTING_STATE_COOLING_DOWN);
                assert_eq!(account.fail_count, Some(1));
                assert_eq!(
                    account.last_routing_error.as_deref(),
                    Some("Cloudflare blocked")
                );

                let peer_account =
                    crate::db::accounts::get_account(&harness.db.pool, peer_account_id)
                        .await
                        .expect("failed to reload post-stream proxy peer account")
                        .expect("post-stream proxy peer account should exist");
                assert_eq!(peer_account.proxy_id, Some(replacement_proxy_id));
            })
        })
        .await;

        let hits = replacement_proxy.recorded_hits().await;
        assert_eq!(hits, vec!["failed-proxy".to_string()]);
        failed_proxy.shutdown();
        replacement_proxy.shutdown();
    }
}
