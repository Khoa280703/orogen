use crate::AppState;
use crate::account::types::{AUTH_MODE_CODEX_OAUTH, AUTH_MODE_GROK_COOKIES};
use crate::account::types::{
    ROUTING_STATE_CANDIDATE, ROUTING_STATE_COOLING_DOWN, ROUTING_STATE_HEALTHY,
};
use crate::providers::types::{ProviderAuthMode, ProviderCapabilities};
use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HealthOverview {
    pub total_accounts: i64,
    pub active_accounts: i64,
    pub total_proxies: i64,
    pub active_proxies: i64,
    pub total_requests_today: i64,
    pub total_requests_week: i64,
    pub error_rate_percent: f64,
    pub active_users_24h: i64,
    pub api_key_count: i64,
    pub provider_verification: Vec<ProviderVerificationGate>,
}

#[derive(Debug, Serialize)]
pub struct ProviderVerificationGate {
    pub provider_slug: String,
    pub provider_name: String,
    pub expected_auth_mode: Option<String>,
    pub has_chat_adapter: bool,
    pub supports_chat_streaming: bool,
    pub supports_responses_api: bool,
    pub active_account_count: i64,
    pub selectable_account_count: i64,
    pub active_public_route_count: i64,
    pub plan_assignment_count: i64,
    pub ready: bool,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
struct ProviderVerificationSnapshot {
    provider_slug: String,
    provider_name: String,
    capabilities: Option<ProviderCapabilities>,
    active_account_count: i64,
    selectable_account_count: i64,
    matching_auth_mode_account_count: i64,
    active_public_route_count: i64,
    plan_assignment_count: i64,
}

/// GET /admin/health - Get system health overview
pub async fn get_health_overview(
    State(state): State<AppState>,
) -> Result<Json<HealthOverview>, (StatusCode, String)> {
    let db = &state.db;

    // Account stats - query separately to avoid tuple issues
    let total_accounts = sqlx::query_scalar::<_, i64>("SELECT COUNT(*)::bigint FROM accounts")
        .fetch_one(db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    let active_accounts = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(CASE WHEN active = true THEN 1 END)::bigint FROM accounts"#,
    )
    .fetch_one(db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    // Proxy stats - query separately
    let total_proxies = sqlx::query_scalar::<_, i64>("SELECT COUNT(*)::bigint FROM proxies")
        .fetch_one(db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    let active_proxies = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(CASE WHEN active = true THEN 1 END)::bigint FROM proxies"#,
    )
    .fetch_one(db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    // Request stats today
    let total_requests_today = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(*)::bigint FROM usage_logs
           WHERE created_at >= NOW() - INTERVAL '1 day'"#,
    )
    .fetch_one(db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    // Request stats week
    let total_requests_week = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(*)::bigint FROM usage_logs
           WHERE created_at >= NOW() - INTERVAL '7 days'"#,
    )
    .fetch_one(db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    // Error rate (last 24h)
    let usage_total = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(*)::bigint FROM usage_logs WHERE created_at >= NOW() - INTERVAL '1 day'"#,
    )
    .fetch_one(db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    let usage_errors = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(*)::bigint FROM usage_logs
           WHERE created_at >= NOW() - INTERVAL '1 day'
             AND status != 'success'"#,
    )
    .fetch_one(db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    let error_rate = if usage_total > 0 {
        (usage_errors as f64 / usage_total as f64) * 100.0
    } else {
        0.0
    };

    // Active users in last 24h (users with usage logs)
    let active_users_24h = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(DISTINCT user_id)::bigint FROM usage_logs
           WHERE created_at >= NOW() - INTERVAL '1 day'"#,
    )
    .fetch_one(db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    // API key count
    let api_key_count = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(*)::bigint FROM api_keys WHERE active = true"#,
    )
    .fetch_one(db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    let provider_rows = sqlx::query_as::<_, (String, String)>(
        r#"SELECT slug, name FROM providers WHERE active = true ORDER BY name ASC"#,
    )
    .fetch_all(db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    let mut provider_verification = Vec::new();
    for (provider_slug, provider_name) in provider_rows {
        let capabilities = state
            .providers
            .chat_provider(&provider_slug)
            .map(|provider| provider.capabilities());
        let expected_auth_mode =
            capabilities.map(|capabilities| provider_auth_mode_slug(capabilities.auth_mode));

        let active_account_count = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*)::bigint FROM accounts WHERE provider_slug = $1 AND active = true"#,
        )
        .bind(&provider_slug)
        .fetch_one(db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

        let matching_auth_mode_account_count = if let Some(expected_auth_mode) = expected_auth_mode
        {
            sqlx::query_scalar::<_, i64>(
                r#"
                SELECT COUNT(*)::bigint
                FROM accounts
                WHERE provider_slug = $1
                  AND active = true
                  AND auth_mode = $2
                "#,
            )
            .bind(&provider_slug)
            .bind(expected_auth_mode)
            .fetch_one(db)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {}", e),
                )
            })?
        } else {
            0
        };

        let selectable_account_count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)::bigint
            FROM accounts
            WHERE provider_slug = $1
              AND active = true
              AND COALESCE(session_status, '') NOT IN ('expired', 'needs_login', 'refresh_failed')
              AND (
                COALESCE(routing_state, 'candidate') IN ($2, $3)
                OR (
                    COALESCE(routing_state, 'candidate') = $4
                    AND cooldown_until IS NOT NULL
                    AND cooldown_until <= NOW()
                )
              )
            "#,
        )
        .bind(&provider_slug)
        .bind(ROUTING_STATE_HEALTHY)
        .bind(ROUTING_STATE_CANDIDATE)
        .bind(ROUTING_STATE_COOLING_DOWN)
        .fetch_one(db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

        let active_public_route_count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)::bigint
            FROM public_model_routes r
            JOIN public_models pm ON pm.id = r.public_model_id
            WHERE r.provider_slug = $1
              AND r.active = true
              AND pm.active = true
            "#,
        )
        .bind(&provider_slug)
        .fetch_one(db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

        let plan_assignment_count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(DISTINCT ppm.plan_id)::bigint
            FROM public_model_routes r
            JOIN public_models pm ON pm.id = r.public_model_id
            JOIN plan_public_models ppm ON ppm.public_model_id = pm.id
            JOIN plans pl ON pl.id = ppm.plan_id
            WHERE r.provider_slug = $1
              AND r.active = true
              AND pm.active = true
              AND pl.active = true
            "#,
        )
        .bind(&provider_slug)
        .fetch_one(db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

        provider_verification.push(build_provider_verification_gate(
            ProviderVerificationSnapshot {
                provider_slug,
                provider_name,
                capabilities,
                active_account_count,
                selectable_account_count,
                matching_auth_mode_account_count,
                active_public_route_count,
                plan_assignment_count,
            },
        ));
    }

    Ok(Json(HealthOverview {
        total_accounts,
        active_accounts,
        total_proxies,
        active_proxies,
        total_requests_today,
        total_requests_week,
        error_rate_percent: error_rate,
        active_users_24h,
        api_key_count,
        provider_verification,
    }))
}

fn provider_auth_mode_slug(auth_mode: ProviderAuthMode) -> &'static str {
    match auth_mode {
        ProviderAuthMode::CookieSession => AUTH_MODE_GROK_COOKIES,
        ProviderAuthMode::OAuthToken => AUTH_MODE_CODEX_OAUTH,
    }
}

fn build_provider_verification_gate(
    snapshot: ProviderVerificationSnapshot,
) -> ProviderVerificationGate {
    let has_chat_adapter = snapshot.capabilities.is_some();
    let expected_auth_mode = snapshot
        .capabilities
        .map(|capabilities| provider_auth_mode_slug(capabilities.auth_mode).to_string());

    let supports_chat_streaming = snapshot
        .capabilities
        .map(|capabilities| capabilities.supports_chat_streaming)
        .unwrap_or(false);
    let supports_responses_api = snapshot
        .capabilities
        .map(|capabilities| capabilities.supports_responses_api)
        .unwrap_or(false);

    let mut warnings = Vec::new();
    if !has_chat_adapter
        && (snapshot.active_public_route_count > 0
            || snapshot.active_account_count > 0
            || snapshot.plan_assignment_count > 0)
    {
        warnings
            .push("Provider has rollout signals but no chat adapter is registered.".to_string());
    }
    if snapshot.active_public_route_count > 0 && snapshot.selectable_account_count == 0 {
        warnings
            .push("Active public routes exist but there are no selectable accounts.".to_string());
    }
    if snapshot.active_account_count > 0 && snapshot.active_public_route_count == 0 {
        warnings.push("Provider has active accounts but no public routes yet.".to_string());
    }
    if snapshot.active_public_route_count > 0 && snapshot.plan_assignment_count == 0 {
        warnings.push("Active public routes exist but no plans currently sell them.".to_string());
    }
    if snapshot.active_account_count > 0
        && has_chat_adapter
        && snapshot.matching_auth_mode_account_count < snapshot.active_account_count
    {
        warnings.push("Some active accounts use an auth_mode that does not match the registered provider adapter.".to_string());
    }

    ProviderVerificationGate {
        provider_slug: snapshot.provider_slug,
        provider_name: snapshot.provider_name,
        expected_auth_mode,
        has_chat_adapter,
        supports_chat_streaming,
        supports_responses_api,
        active_account_count: snapshot.active_account_count,
        selectable_account_count: snapshot.selectable_account_count,
        active_public_route_count: snapshot.active_public_route_count,
        plan_assignment_count: snapshot.plan_assignment_count,
        ready: warnings.is_empty(),
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use crate::providers::types::{ProviderAuthMode, ProviderCapabilities};

    use super::{ProviderVerificationSnapshot, build_provider_verification_gate};

    #[test]
    fn flags_missing_accounts_for_routed_provider() {
        let gate = build_provider_verification_gate(ProviderVerificationSnapshot {
            provider_slug: "codex".to_string(),
            provider_name: "Codex".to_string(),
            capabilities: Some(ProviderCapabilities {
                auth_mode: ProviderAuthMode::OAuthToken,
                supports_chat_streaming: true,
                supports_proxy: true,
                supports_responses_api: true,
            }),
            active_account_count: 0,
            selectable_account_count: 0,
            matching_auth_mode_account_count: 0,
            active_public_route_count: 2,
            plan_assignment_count: 1,
        });

        assert!(!gate.ready);
        assert!(
            gate.warnings
                .iter()
                .any(|warning| warning.contains("no selectable accounts"))
        );
    }

    #[test]
    fn flags_auth_mode_mismatch() {
        let gate = build_provider_verification_gate(ProviderVerificationSnapshot {
            provider_slug: "codex".to_string(),
            provider_name: "Codex".to_string(),
            capabilities: Some(ProviderCapabilities {
                auth_mode: ProviderAuthMode::OAuthToken,
                supports_chat_streaming: true,
                supports_proxy: true,
                supports_responses_api: true,
            }),
            active_account_count: 3,
            selectable_account_count: 3,
            matching_auth_mode_account_count: 1,
            active_public_route_count: 1,
            plan_assignment_count: 1,
        });

        assert!(!gate.ready);
        assert!(
            gate.warnings
                .iter()
                .any(|warning| warning.contains("auth_mode"))
        );
    }

    #[test]
    fn marks_provider_ready_when_gates_are_clear() {
        let gate = build_provider_verification_gate(ProviderVerificationSnapshot {
            provider_slug: "codex".to_string(),
            provider_name: "Codex".to_string(),
            capabilities: Some(ProviderCapabilities {
                auth_mode: ProviderAuthMode::OAuthToken,
                supports_chat_streaming: true,
                supports_proxy: true,
                supports_responses_api: true,
            }),
            active_account_count: 2,
            selectable_account_count: 2,
            matching_auth_mode_account_count: 2,
            active_public_route_count: 2,
            plan_assignment_count: 1,
        });

        assert!(gate.ready);
        assert!(gate.warnings.is_empty());
        assert_eq!(gate.expected_auth_mode.as_deref(), Some("codex_oauth"));
        assert!(gate.supports_responses_api);
    }

    #[test]
    fn flags_accounts_without_public_routes() {
        let gate = build_provider_verification_gate(ProviderVerificationSnapshot {
            provider_slug: "codex".to_string(),
            provider_name: "Codex".to_string(),
            capabilities: Some(ProviderCapabilities {
                auth_mode: ProviderAuthMode::OAuthToken,
                supports_chat_streaming: true,
                supports_proxy: true,
                supports_responses_api: true,
            }),
            active_account_count: 2,
            selectable_account_count: 2,
            matching_auth_mode_account_count: 2,
            active_public_route_count: 0,
            plan_assignment_count: 0,
        });

        assert!(!gate.ready);
        assert!(
            gate.warnings
                .iter()
                .any(|warning| warning.contains("no public routes"))
        );
    }
}
