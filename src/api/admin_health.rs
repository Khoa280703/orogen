use crate::AppState;
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
    }))
}
