use axum::Json;
use axum::extract::Query;
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::db::usage_logs;

#[derive(Debug, Deserialize)]
pub struct UsageQuery {
    #[serde(default = "default_days")]
    pub days: i32,
}

#[derive(Debug, Deserialize)]
pub struct UsageLogsQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
}

fn default_days() -> i32 {
    7
}

fn default_page() -> u32 {
    1
}

fn default_limit() -> u32 {
    20
}

#[derive(Debug, Serialize)]
pub struct StatsOverview {
    pub total_accounts: i64,
    pub active_accounts: i64,
    pub requests_today: i64,
    pub errors_today: i64,
    pub total_conversations: i64,
    pub total_image_generations: i64,
}

#[derive(Debug, Serialize)]
pub struct DailyUsage {
    pub day: String,
    pub total: i64,
    pub success: i64,
    pub errors: i64,
}

/// Get stats overview
pub async fn get_stats_overview(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<StatsOverview>, axum::http::StatusCode> {
    let db = &state.db;
    let (total, active, requests, errors) = usage_logs::get_stats_overview(&db)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    let total_conversations: i64 =
        sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*)::BIGINT FROM conversations WHERE active = true"#,
        )
            .fetch_one(db)
            .await
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    let total_image_generations: i64 =
        sqlx::query_scalar::<_, i64>(r#"SELECT COUNT(*)::BIGINT FROM image_generations"#)
            .fetch_one(db)
            .await
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(StatsOverview {
        total_accounts: total,
        active_accounts: active,
        requests_today: requests,
        errors_today: errors,
        total_conversations,
        total_image_generations,
    }))
}

/// Get daily usage breakdown
pub async fn get_daily_usage(
    axum::extract::State(state): axum::extract::State<AppState>,
    Query(params): Query<UsageQuery>,
) -> Result<Json<Vec<DailyUsage>>, axum::http::StatusCode> {
    let db = &state.db;
    let usage = usage_logs::get_daily_usage(&db, params.days)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = usage
        .into_iter()
        .map(|u| DailyUsage {
            day: u.day.unwrap_or_default(),
            total: u.total.unwrap_or(0),
            success: u.success.unwrap_or(0),
            errors: u.total.unwrap_or(0) - u.success.unwrap_or(0),
        })
        .collect();

    Ok(Json(response))
}

/// Get usage logs with pagination
pub async fn get_usage_logs(
    axum::extract::State(state): axum::extract::State<AppState>,
    Query(params): Query<UsageLogsQuery>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let db = &state.db;

    let page = params.page.max(1);
    let limit = params.limit.clamp(1, 100) as i64;
    let offset = (page.saturating_sub(1) as i64) * limit;
    let logs = usage_logs::get_usage_logs(
        &db,
        offset,
        limit,
        params.search.as_deref(),
        params.status.as_deref(),
        params.model.as_deref(),
    )
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let count = usage_logs::get_usage_log_count(
        &db,
        params.search.as_deref(),
        params.status.as_deref(),
        params.model.as_deref(),
    )
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "logs": logs,
        "total": count,
        "offset": offset,
        "limit": limit,
        "page": page
    })))
}
