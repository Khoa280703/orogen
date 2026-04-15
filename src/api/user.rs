use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::middleware::jwt_auth::JwtUser;

// ==================== GET /user/me ====================

#[derive(Debug, Serialize)]
pub struct UserProfileResponse {
    pub user: UserDetail,
    pub plan: Option<PlanDetail>,
    pub balance: BalanceDetail,
}

#[derive(Debug, Serialize)]
pub struct UserDetail {
    pub id: i32,
    pub email: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub locale: String,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct PlanDetail {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub requests_per_day: Option<i32>,
    pub requests_per_month: Option<i32>,
    pub price_usd: Option<String>,
    pub price_vnd: Option<i32>,
    pub features: Option<serde_json::Value>,
    pub starts_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct BalanceDetail {
    pub id: i32,
    pub amount: String,
    pub updated_at: DateTime<Utc>,
}

pub async fn get_user_profile(
    State(state): State<AppState>,
    user: axum::extract::Extension<JwtUser>,
) -> Result<Json<UserProfileResponse>, (StatusCode, String)> {
    let db = &state.db;

    // Get user
    let user_detail = crate::db::users::get_user(db, user.user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;

    // Get active plan
    let plan = crate::db::user_plans::get_active_plan(db, user.user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    let plan_detail = if let Some(up) = &plan {
        if let Ok(Some(p)) = crate::db::plans::get_plan(db, up.plan_id).await {
            Some(PlanDetail {
                id: p.id,
                name: p.name,
                slug: p.slug,
                requests_per_day: p.requests_per_day,
                requests_per_month: p.requests_per_month,
                price_usd: p.price_usd,
                price_vnd: p.price_vnd,
                features: p.features,
                starts_at: up.starts_at.unwrap_or(Utc::now()),
                expires_at: up.expires_at,
            })
        } else {
            None
        }
    } else {
        None
    };

    // Get balance
    let balance = crate::db::balances::get_balance(db, user.user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "Balance not found".to_string()))?;

    Ok(Json(UserProfileResponse {
        user: UserDetail {
            id: user_detail.id,
            email: user_detail.email,
            name: user_detail.name,
            avatar_url: user_detail.avatar_url,
            locale: user_detail.locale,
            created_at: user_detail.created_at,
        },
        plan: plan_detail,
        balance: BalanceDetail {
            id: balance.id,
            amount: balance.amount,
            updated_at: balance.updated_at,
        },
    }))
}

// ==================== GET /user/keys ====================

#[derive(Debug, Serialize)]
pub struct ApiKeyListResponse {
    pub keys: Vec<ApiKeySummary>,
}

#[derive(Debug, Serialize)]
pub struct ApiKeySummary {
    pub id: i32,
    pub label: String,
    pub key_prefix: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub active: bool,
}

pub async fn list_user_api_keys(
    State(state): State<AppState>,
    user: axum::extract::Extension<JwtUser>,
) -> Result<Json<ApiKeyListResponse>, (StatusCode, String)> {
    let db = &state.db;

    let keys = crate::db::api_keys::list_by_user(db, user.user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    let key_summaries: Vec<ApiKeySummary> = keys
        .into_iter()
        .map(|k| ApiKeySummary {
            id: k.id,
            label: k.label,
            key_prefix: k.key.chars().take(8).collect(),
            created_at: k.created_at,
            last_used_at: k.last_used_at,
            active: k.active,
        })
        .collect();

    Ok(Json(ApiKeyListResponse {
        keys: key_summaries,
    }))
}

// ==================== POST /user/keys ====================

#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub label: String,
}

#[derive(Debug, Serialize)]
pub struct CreateApiKeyResponse {
    pub id: i32,
    pub label: String,
    pub key: String,
    pub created_at: DateTime<Utc>,
}

pub async fn create_user_api_key(
    State(state): State<AppState>,
    user: axum::extract::Extension<JwtUser>,
    Json(payload): Json<CreateApiKeyRequest>,
) -> Result<Json<CreateApiKeyResponse>, (StatusCode, String)> {
    let db = &state.db;

    let key = crate::db::api_keys::create_key(db, &payload.label, Some(user.user_id))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    // Add to in-memory store
    state.api_keys.write().await.insert(key.key.clone());

    Ok(Json(CreateApiKeyResponse {
        id: key.id,
        label: key.label,
        key: key.key,
        created_at: key.created_at,
    }))
}

// ==================== DELETE /user/keys/:id ====================

pub async fn revoke_user_api_key(
    Path(id): Path<i32>,
    State(state): State<AppState>,
    user: axum::extract::Extension<JwtUser>,
) -> Result<StatusCode, (StatusCode, String)> {
    let db = &state.db;

    // Verify ownership
    let key = crate::db::api_keys::get_key(db, id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "API key not found".to_string()))?;

    if let Some(key_user_id) = key.user_id {
        if key_user_id != user.user_id {
            return Err((
                StatusCode::FORBIDDEN,
                "Cannot revoke another user's key".to_string(),
            ));
        }
    }

    // Revoke key
    crate::db::api_keys::revoke_key(db, id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    // Remove from in-memory store
    state.api_keys.write().await.remove(&key.key);

    Ok(StatusCode::NO_CONTENT)
}

// ==================== GET /user/usage ====================

#[derive(Debug, Deserialize)]
pub struct UsageQueryParams {
    pub days: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct UsageStatsResponse {
    pub total_requests: i64,
    pub today_requests: i64,
    pub daily_stats: Vec<DailyUsage>,
}

#[derive(Debug, Serialize)]
pub struct DailyUsage {
    pub date: String,
    pub requests: i64,
    pub success: i64,
    pub failed: i64,
}

pub async fn get_user_usage(
    State(state): State<AppState>,
    user: axum::extract::Extension<JwtUser>,
    Query(params): Query<UsageQueryParams>,
) -> Result<Json<UsageStatsResponse>, (StatusCode, String)> {
    let db = &state.db;

    let days = params.days.unwrap_or(7);
    let start_date = Utc::now() - chrono::Duration::days(days as i64);

    // Get usage logs for user
    let logs = crate::db::usage_logs::list_by_user(db, user.user_id, Some(start_date.naive_utc()))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    // Calculate stats
    let total_requests = logs.len() as i64;
    let today = Utc::now().date_naive();

    let today_requests = logs
        .iter()
        .filter(|log| log.created_at.date_naive() == today)
        .count() as i64;

    // Group by day
    let mut daily_map: std::collections::HashMap<chrono::NaiveDate, DailyUsage> =
        std::collections::HashMap::new();

    for log in &logs {
        let date = log.created_at.date_naive();
        let entry = daily_map.entry(date).or_insert(DailyUsage {
            date: date.format("%Y-%m-%d").to_string(),
            requests: 0,
            success: 0,
            failed: 0,
        });
        entry.requests += 1;
        if log.status_code >= 200 && log.status_code < 400 {
            entry.success += 1;
        } else {
            entry.failed += 1;
        }
    }

    let mut daily_stats: Vec<DailyUsage> = daily_map.into_values().collect();
    daily_stats.sort_by(|a, b| a.date.cmp(&b.date));

    Ok(Json(UsageStatsResponse {
        total_requests,
        today_requests,
        daily_stats,
    }))
}

// ==================== GET /user/billing ====================

#[derive(Debug, Serialize)]
pub struct BillingResponse {
    pub balance: BalanceDetail,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Serialize)]
pub struct Transaction {
    pub id: i32,
    pub amount: String,
    pub r#type: String,
    pub reference: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub async fn get_user_billing(
    State(state): State<AppState>,
    user: axum::extract::Extension<JwtUser>,
) -> Result<Json<BillingResponse>, (StatusCode, String)> {
    let db = &state.db;

    // Get balance
    let balance = crate::db::balances::get_balance(db, user.user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "Balance not found".to_string()))?;

    // Get transactions (if table exists)
    let transactions = vec![]; // TODO: Implement transactions table

    Ok(Json(BillingResponse {
        balance: BalanceDetail {
            id: balance.id,
            amount: balance.amount,
            updated_at: balance.updated_at,
        },
        transactions,
    }))
}
