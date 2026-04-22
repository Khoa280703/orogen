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
    #[serde(default)]
    pub provider: Option<String>,
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

#[derive(Debug, Serialize)]
pub struct UsageLogFilterOptions {
    pub statuses: Vec<String>,
    pub models: Vec<String>,
    pub providers: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct UsageLogAggregatesResponse {
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub cached_tokens: i64,
    pub credits_used: i64,
}

#[derive(Debug, Serialize)]
pub struct UsageLogBreakdownRowResponse {
    pub label: String,
    pub requests: i64,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub cached_tokens: i64,
    pub credits_used: i64,
}

/// Get stats overview
pub async fn get_stats_overview(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<StatsOverview>, axum::http::StatusCode> {
    let db = &state.db;
    let (total, active, requests, errors) = usage_logs::get_stats_overview(&db)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    let total_conversations: i64 = sqlx::query_scalar::<_, i64>(
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
        params.provider.as_deref(),
    )
    .await
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let count = usage_logs::get_usage_log_count(
        &db,
        params.search.as_deref(),
        params.status.as_deref(),
        params.model.as_deref(),
        params.provider.as_deref(),
    )
    .await
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let aggregates = usage_logs::get_usage_log_aggregates(
        &db,
        params.search.as_deref(),
        params.status.as_deref(),
        params.model.as_deref(),
        params.provider.as_deref(),
    )
    .await
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let provider_breakdown = usage_logs::get_provider_usage_breakdown(
        &db,
        params.search.as_deref(),
        params.status.as_deref(),
        params.model.as_deref(),
        params.provider.as_deref(),
        8,
    )
    .await
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let model_breakdown = usage_logs::get_model_usage_breakdown(
        &db,
        params.search.as_deref(),
        params.status.as_deref(),
        params.model.as_deref(),
        params.provider.as_deref(),
        8,
    )
    .await
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let status_options = usage_logs::list_usage_log_statuses(&db)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    let model_options = usage_logs::list_usage_log_models(&db)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    let provider_options = usage_logs::list_usage_log_providers(&db)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "logs": logs,
        "total": count,
        "offset": offset,
        "limit": limit,
        "page": page,
        "filters": UsageLogFilterOptions {
            statuses: status_options,
            models: model_options,
            providers: provider_options,
        },
        "aggregates": UsageLogAggregatesResponse {
            prompt_tokens: aggregates.prompt_tokens.unwrap_or(0),
            completion_tokens: aggregates.completion_tokens.unwrap_or(0),
            cached_tokens: aggregates.cached_tokens.unwrap_or(0),
            credits_used: aggregates.credits_used.unwrap_or(0),
        },
        "breakdowns": {
            "providers": provider_breakdown.into_iter().map(|row| UsageLogBreakdownRowResponse {
                label: row.label.unwrap_or_else(|| "unknown".to_string()),
                requests: row.requests.unwrap_or(0),
                prompt_tokens: row.prompt_tokens.unwrap_or(0),
                completion_tokens: row.completion_tokens.unwrap_or(0),
                cached_tokens: row.cached_tokens.unwrap_or(0),
                credits_used: row.credits_used.unwrap_or(0),
            }).collect::<Vec<_>>(),
            "models": model_breakdown.into_iter().map(|row| UsageLogBreakdownRowResponse {
                label: row.label.unwrap_or_else(|| "unknown".to_string()),
                requests: row.requests.unwrap_or(0),
                prompt_tokens: row.prompt_tokens.unwrap_or(0),
                completion_tokens: row.completion_tokens.unwrap_or(0),
                cached_tokens: row.cached_tokens.unwrap_or(0),
                credits_used: row.credits_used.unwrap_or(0),
            }).collect::<Vec<_>>(),
        },
    })))
}
