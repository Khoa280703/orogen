use axum::Json;
use axum::extract::Path;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::db::api_keys;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiKeyCreateRequest {
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub quota_per_day: Option<i32>,
    #[serde(default)]
    pub daily_credit_limit: Option<i64>,
    #[serde(default)]
    pub monthly_credit_limit: Option<i64>,
    #[serde(default)]
    pub max_input_tokens: Option<i32>,
    #[serde(default)]
    pub max_output_tokens: Option<i32>,
    #[serde(default)]
    pub plan_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiKeyUpdateRequest {
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub active: Option<bool>,
    #[serde(default)]
    pub quota_per_day: Option<i32>,
    #[serde(default)]
    pub daily_credit_limit: Option<i64>,
    #[serde(default)]
    pub monthly_credit_limit: Option<i64>,
    #[serde(default)]
    pub max_input_tokens: Option<i32>,
    #[serde(default)]
    pub max_output_tokens: Option<i32>,
    #[serde(default)]
    pub plan_id: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyResponse {
    pub id: i32,
    pub key: String,
    pub label: Option<String>,
    pub active: bool,
    pub quota_per_day: Option<i32>,
    pub daily_credit_limit: Option<i64>,
    pub monthly_credit_limit: Option<i64>,
    pub max_input_tokens: Option<i32>,
    pub max_output_tokens: Option<i32>,
    pub plan_id: Option<i32>,
    pub plan_name: Option<String>,
    pub created_at: Option<String>,
}

/// Generate a random API key
fn generate_api_key() -> String {
    let charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::rng();
    (0..32)
        .map(|_| {
            charset
                .chars()
                .nth(rng.random_range(0..charset.len()))
                .unwrap()
        })
        .collect()
}

/// List all API keys
pub async fn list_api_keys(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<Vec<ApiKeyResponse>>, axum::http::StatusCode> {
    let db = &state.db;
    let keys = api_keys::list_api_keys(&db)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = keys
        .into_iter()
        .map(|key| ApiKeyResponse {
            id: key.id,
            key: key.key,
            label: key.label,
            active: key.active.unwrap_or(true),
            quota_per_day: key.quota_per_day,
            daily_credit_limit: key.daily_credit_limit,
            monthly_credit_limit: key.monthly_credit_limit,
            max_input_tokens: key.max_input_tokens,
            max_output_tokens: key.max_output_tokens,
            plan_id: key.plan_id,
            plan_name: key.plan_name,
            created_at: key.created_at.map(|d| d.to_rfc3339()),
        })
        .collect();

    Ok(Json(response))
}

/// Create a new API key
pub async fn create_api_key(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(req): Json<ApiKeyCreateRequest>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let db = &state.db;
    let key = generate_api_key();
    if let Some(plan_id) = req.plan_id {
        let plan_exists = crate::db::plans::get_plan(db, plan_id)
            .await
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
            .is_some();
        if !plan_exists {
            return Err(axum::http::StatusCode::BAD_REQUEST);
        }
    }

    let id = api_keys::create_api_key(
        &db,
        &key,
        req.label.as_deref(),
        req.quota_per_day,
        req.daily_credit_limit,
        req.monthly_credit_limit,
        req.max_input_tokens,
        req.max_output_tokens,
        req.plan_id,
    )
    .await
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    state.api_keys.write().await.insert(key.clone());

    // Return full key only on create
    Ok(Json(serde_json::json!({
        "id": id,
        "key": key
    })))
}

/// Update an API key
pub async fn update_api_key(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<ApiKeyUpdateRequest>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let db = &state.db;
    let existing_key = if req.active.is_some() {
        api_keys::get_api_key(&db, id)
            .await
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
            .map(|key| key.key)
    } else {
        None
    };

    if let Some(plan_id) = req.plan_id {
        let plan_exists = crate::db::plans::get_plan(db, plan_id)
            .await
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
            .is_some();
        if !plan_exists {
            return Err(axum::http::StatusCode::BAD_REQUEST);
        }
    }

    api_keys::update_api_key(
        &db,
        id,
        req.label.as_deref(),
        req.active,
        req.quota_per_day,
        req.daily_credit_limit,
        req.monthly_credit_limit,
        req.max_input_tokens,
        req.max_output_tokens,
        req.plan_id,
    )
    .await
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(key) = existing_key {
        let mut keys = state.api_keys.write().await;
        if req.active == Some(false) {
            keys.remove(&key);
        } else if req.active == Some(true) {
            keys.insert(key);
        }
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

/// Delete an API key
pub async fn delete_api_key(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let db = &state.db;
    let key_to_remove = api_keys::get_api_key(&db, id)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
        .map(|key| key.key);

    api_keys::delete_api_key(&db, id)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(key) = key_to_remove {
        state.api_keys.write().await.remove(&key);
    }

    Ok(Json(serde_json::json!({ "success": true })))
}
