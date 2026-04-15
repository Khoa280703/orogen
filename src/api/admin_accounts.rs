use axum::Json;
use axum::extract::Path;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::AppState;
use crate::account::types::GrokCookies;
use crate::db::{account_sessions, accounts};
use crate::services::grok_profile_browser;

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountCreateRequest {
    pub name: String,
    #[serde(default)]
    pub cookies: Option<Value>,
    #[serde(default)]
    pub proxy_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountUpdateRequest {
    #[serde(default)]
    pub cookies: Option<Value>,
    #[serde(default)]
    pub active: Option<bool>,
    #[serde(default)]
    pub proxy_id: Option<Option<i32>>,
}

#[derive(Debug, Serialize)]
pub struct AccountResponse {
    pub id: i32,
    pub name: String,
    pub cookies: Value,
    pub active: bool,
    pub proxy_id: Option<i32>,
    pub profile_dir: Option<String>,
    pub session_status: String,
    pub session_error: Option<String>,
    pub request_count: i64,
    pub fail_count: i32,
    pub success_count: i64,
    pub last_used: Option<String>,
    pub created_at: Option<String>,
    pub session_checked_at: Option<String>,
    pub cookies_synced_at: Option<String>,
}

pub async fn list_accounts(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<Vec<AccountResponse>>, StatusCode> {
    let accounts_list = accounts::list_accounts(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = accounts_list
        .into_iter()
        .map(|acc| AccountResponse {
            id: acc.id,
            name: acc.name,
            cookies: acc.cookies,
            active: acc.active.unwrap_or(true),
            proxy_id: acc.proxy_id,
            profile_dir: acc.profile_dir,
            session_status: acc
                .session_status
                .unwrap_or_else(|| account_sessions::SESSION_STATUS_UNKNOWN.to_string()),
            session_error: acc.session_error,
            request_count: acc.request_count.unwrap_or(0),
            fail_count: acc.fail_count.unwrap_or(0),
            success_count: acc.success_count.unwrap_or(0),
            last_used: acc.last_used.map(|d| d.to_rfc3339()),
            created_at: acc.created_at.map(|d| d.to_rfc3339()),
            session_checked_at: acc.session_checked_at.map(|d| d.to_rfc3339()),
            cookies_synced_at: acc.cookies_synced_at.map(|d| d.to_rfc3339()),
        })
        .collect();

    Ok(Json(response))
}

pub async fn create_account(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(req): Json<AccountCreateRequest>,
) -> Result<Json<Value>, StatusCode> {
    let cookies = match req.cookies.as_ref() {
        Some(value) => normalize_cookies(value)?,
        None => json!({}),
    };
    let profile_dir = grok_profile_browser::resolve_profile_dir(&req.name)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let profile_dir = profile_dir.to_string_lossy().to_string();

    let id = accounts::create_account(
        &state.db,
        &req.name,
        &cookies,
        req.proxy_id,
        Some(&profile_dir),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !has_usable_cookies(&cookies) {
        account_sessions::mark_needs_login(&state.db, id, &profile_dir)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Json(json!({ "id": id })))
}

pub async fn update_account(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<AccountUpdateRequest>,
) -> Result<Json<Value>, StatusCode> {
    let cookies = req.cookies.as_ref().map(normalize_cookies).transpose()?;

    accounts::update_account(&state.db, id, cookies.as_ref(), req.active, req.proxy_id, None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({ "success": true })))
}

pub async fn delete_account(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, StatusCode> {
    accounts::delete_account(&state.db, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({ "success": true })))
}

pub async fn open_login_browser(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let account = accounts::get_account(&state.db, id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load account".to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Account not found".to_string()))?;

    let result = grok_profile_browser::launch_login_browser(&account.name)
        .await
        .map_err(|error| {
            tracing::warn!(account_id = id, %error, "Failed to launch account login browser");
            (StatusCode::INTERNAL_SERVER_ERROR, error)
        })?;

    account_sessions::mark_needs_login(&state.db, id, &result.profile_dir)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save profile directory".to_string()))?;

    Ok(Json(json!({
        "success": true,
        "profile_dir": result.profile_dir,
        "pid": result.pid,
        "message": result.message.unwrap_or_else(|| "Browser launched.".to_string())
    })))
}

pub async fn sync_profile(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let account = accounts::get_account(&state.db, id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load account".to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Account not found".to_string()))?;

    let sync_result = match grok_profile_browser::sync_profile_cookies(&account.name).await {
        Ok(result) => result,
        Err(error) => {
            tracing::warn!(account_id = id, %error, "Failed to sync cookies from browser profile");
            let _ = account_sessions::mark_profile_sync_error(
                &state.db,
                id,
                account.profile_dir.as_deref(),
                &error,
            )
            .await;
            return Err((StatusCode::BAD_GATEWAY, error));
        }
    };

    let cookies = serde_json::to_value(&sync_result.cookies)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to serialize synced cookies".to_string()))?;
    account_sessions::mark_profile_sync_success(&state.db, id, &cookies, &sync_result.profile_dir)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to persist synced cookies".to_string()))?;

    Ok(Json(json!({
        "success": true,
        "profile_dir": sync_result.profile_dir,
        "message": sync_result.message.unwrap_or_else(|| "Cookies synced from browser profile.".to_string())
    })))
}

fn normalize_cookies(value: &Value) -> Result<Value, StatusCode> {
    let cookies = GrokCookies::from_value(value).map_err(|error| {
        tracing::warn!(%error, "Invalid account cookies payload");
        StatusCode::BAD_REQUEST
    })?;

    serde_json::to_value(cookies).map_err(|error| {
        tracing::warn!(%error, "Failed to serialize normalized cookies");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

fn has_usable_cookies(value: &Value) -> bool {
    value
        .as_object()
        .and_then(|obj| obj.get("sso"))
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().is_empty())
}
