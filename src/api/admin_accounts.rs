use axum::Json;
use axum::extract::Path;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::AppState;
use crate::account::types::{
    AUTH_MODE_CODEX_OAUTH, AUTH_MODE_GROK_COOKIES, AccountCredential,
    CREDENTIAL_TYPE_CODEX_OAUTH_TOKENS, CREDENTIAL_TYPE_GROK_COOKIES, PROVIDER_CODEX,
    PROVIDER_GROK,
};
use crate::db::{account_credentials, account_sessions, accounts};
use crate::services::{codex_oauth, grok_profile_browser, provider_account_usage};

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountCreateRequest {
    pub name: String,
    #[serde(default)]
    pub provider_slug: Option<String>,
    #[serde(default)]
    pub credentials: Option<Value>,
    #[serde(default)]
    pub cookies: Option<Value>,
    #[serde(default)]
    pub proxy_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountUpdateRequest {
    #[serde(default)]
    pub credentials: Option<Value>,
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
    pub provider_slug: String,
    pub credential_preview: Value,
    pub account_label: Option<String>,
    pub external_account_id: Option<String>,
    pub auth_mode: Option<String>,
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
    pub routing_state: String,
    pub cooldown_until: Option<String>,
    pub last_routing_error: Option<String>,
    pub rate_limit_streak: i32,
    pub auth_failure_streak: i32,
    pub refresh_failure_streak: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodexManualCallbackRequest {
    pub callback_url: String,
}

pub async fn list_accounts(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<Vec<AccountResponse>>, StatusCode> {
    let accounts_list = accounts::list_accounts(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = accounts_list
        .into_iter()
        .map(|acc| {
            let credential_preview = credential_preview_for_row(&acc);
            AccountResponse {
                id: acc.id,
                name: acc.name,
                provider_slug: acc.provider_slug,
                credential_preview,
                account_label: acc.account_label,
                external_account_id: acc.external_account_id,
                auth_mode: acc.auth_mode,
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
                routing_state: acc.routing_state,
                cooldown_until: acc.cooldown_until.map(|d| d.to_rfc3339()),
                last_routing_error: acc.last_routing_error,
                rate_limit_streak: acc.rate_limit_streak,
                auth_failure_streak: acc.auth_failure_streak,
                refresh_failure_streak: acc.refresh_failure_streak,
            }
        })
        .collect();

    Ok(Json(response))
}

pub async fn create_account(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(req): Json<AccountCreateRequest>,
) -> Result<Json<Value>, StatusCode> {
    let provider_slug = normalize_provider_slug(req.provider_slug.as_deref())?;
    let credential_input = req.credentials.as_ref().or(req.cookies.as_ref());
    let credential = credential_input
        .map(|value| normalize_account_credential(&provider_slug, value))
        .transpose()?;

    let profile_dir = if provider_slug == PROVIDER_GROK {
        Some(
            grok_profile_browser::resolve_profile_dir(&req.name)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .to_string_lossy()
                .to_string(),
        )
    } else {
        None
    };

    let legacy_cookies = match credential.as_ref() {
        Some(AccountCredential::GrokCookies(cookies)) => {
            serde_json::to_value(cookies).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
        _ => json!({}),
    };

    let id = accounts::create_account(
        &state.db,
        &req.name,
        &provider_slug,
        &legacy_cookies,
        req.proxy_id,
        profile_dir.as_deref(),
        Some(match provider_slug.as_str() {
            PROVIDER_GROK => AUTH_MODE_GROK_COOKIES,
            PROVIDER_CODEX => AUTH_MODE_CODEX_OAUTH,
            _ => AUTH_MODE_GROK_COOKIES,
        }),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(credential) = credential {
        persist_account_credential(&state, id, &provider_slug, &credential)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    } else if provider_slug == PROVIDER_GROK {
        account_sessions::mark_needs_login(
            &state.db,
            id,
            profile_dir.as_deref().unwrap_or_default(),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    } else {
        mark_codex_needs_login(&state.db, id, "Codex login required before first use.")
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
    let account = accounts::get_account(&state.db, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let provider_slug = account.provider_slug.clone();
    let credential_input = req.credentials.as_ref().or(req.cookies.as_ref());
    let credential = credential_input
        .map(|value| normalize_account_credential(&provider_slug, value))
        .transpose()?;
    let legacy_cookies = credential.as_ref().and_then(|credential| match credential {
        AccountCredential::GrokCookies(cookies) => serde_json::to_value(cookies).ok(),
        _ => None,
    });

    accounts::update_account(
        &state.db,
        id,
        legacy_cookies.as_ref(),
        req.active,
        req.proxy_id,
        None,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(credential) = credential {
        persist_account_credential(&state, id, &provider_slug, &credential)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Json(json!({ "success": true })))
}

pub async fn delete_account(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, StatusCode> {
    account_credentials::delete_account_credentials(&state.db, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    accounts::delete_account(&state.db, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({ "success": true })))
}

pub async fn get_account_usage(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<provider_account_usage::ProviderAccountUsage>, (StatusCode, String)> {
    let account = accounts::get_account(&state.db, id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load account".to_string(),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "Account not found".to_string()))?;

    let usage = provider_account_usage::fetch_account_usage(&state.db, &state.config, &account)
        .await
        .map_err(|error| (StatusCode::BAD_GATEWAY, error))?;

    Ok(Json(usage))
}

pub async fn open_login_browser(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let account = accounts::get_account(&state.db, id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load account".to_string(),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "Account not found".to_string()))?;

    if account.provider_slug != PROVIDER_GROK {
        return Err((
            StatusCode::BAD_REQUEST,
            "Open login browser is only available for Grok accounts".to_string(),
        ));
    }

    let result = grok_profile_browser::launch_login_browser(&account.name)
        .await
        .map_err(|error| {
            tracing::warn!(account_id = id, %error, "Failed to launch account login browser");
            (StatusCode::INTERNAL_SERVER_ERROR, error)
        })?;

    account_sessions::mark_needs_login(&state.db, id, &result.profile_dir)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to save profile directory".to_string(),
            )
        })?;

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
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load account".to_string(),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "Account not found".to_string()))?;

    if account.provider_slug != PROVIDER_GROK {
        return Err((
            StatusCode::BAD_REQUEST,
            "Sync profile is only available for Grok accounts".to_string(),
        ));
    }

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

    let cookies = serde_json::to_value(&sync_result.cookies).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to serialize synced cookies".to_string(),
        )
    })?;
    account_sessions::mark_profile_sync_success(&state.db, id, &cookies, &sync_result.profile_dir)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to persist synced cookies".to_string(),
            )
        })?;

    Ok(Json(json!({
        "success": true,
        "profile_dir": sync_result.profile_dir,
        "message": sync_result.message.unwrap_or_else(|| "Cookies synced from browser profile.".to_string())
    })))
}

pub async fn start_codex_login(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let account = accounts::get_account(&state.db, id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load account".to_string(),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "Account not found".to_string()))?;

    if account.provider_slug != PROVIDER_CODEX {
        return Err((
            StatusCode::BAD_REQUEST,
            "Codex native login is only available for Codex accounts".to_string(),
        ));
    }

    let session =
        codex_oauth::start_device_login(&state.db, &state.config, &state.codex_login_sessions, id)
            .await
            .map_err(|error| (StatusCode::BAD_REQUEST, error))?;

    Ok(Json(json!({ "success": true, "session": session })))
}

pub async fn get_codex_login_status(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let account = accounts::get_account(&state.db, id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load account".to_string(),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "Account not found".to_string()))?;

    if account.provider_slug != PROVIDER_CODEX {
        return Err((
            StatusCode::BAD_REQUEST,
            "Codex native login status is only available for Codex accounts".to_string(),
        ));
    }

    let session = codex_oauth::get_login_status_for_account(&state.codex_login_sessions, id)
        .await
        .ok_or((
            StatusCode::NOT_FOUND,
            "No active Codex login session for this account".to_string(),
        ))?;

    Ok(Json(json!({ "success": true, "session": session })))
}

pub async fn submit_codex_manual_callback(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<CodexManualCallbackRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let account = accounts::get_account(&state.db, id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load account".to_string(),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "Account not found".to_string()))?;

    if account.provider_slug != PROVIDER_CODEX {
        return Err((
            StatusCode::BAD_REQUEST,
            "Manual callback submit is only available for Codex accounts".to_string(),
        ));
    }

    let session =
        codex_oauth::submit_manual_callback_url(&state.codex_login_sessions, id, &req.callback_url)
            .await
            .map_err(|error| (StatusCode::BAD_REQUEST, error))?;

    Ok(Json(json!({ "success": true, "session": session })))
}

pub async fn refresh_codex_account(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let account = accounts::get_account(&state.db, id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load account".to_string(),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "Account not found".to_string()))?;

    if account.provider_slug != PROVIDER_CODEX {
        return Err((
            StatusCode::BAD_REQUEST,
            "Refresh token is only available for Codex accounts".to_string(),
        ));
    }

    let credential = normalize_existing_credential(&account)
        .map_err(|error| (StatusCode::BAD_REQUEST, error))?;
    let tokens = credential.as_codex_tokens().cloned().ok_or((
        StatusCode::BAD_REQUEST,
        "Codex account tokens not found".to_string(),
    ))?;

    let refreshed =
        match codex_oauth::refresh_account_tokens(&state.db, &state.config, id, &tokens).await {
            Ok(tokens) => tokens,
            Err(error) => {
                let _ = codex_oauth::mark_refresh_failed(&state.db, id, &error).await;
                return Err((StatusCode::BAD_GATEWAY, error));
            }
        };

    Ok(Json(json!({
        "success": true,
        "credential_preview": refreshed.to_preview(),
        "message": "Codex token refreshed successfully.",
    })))
}

fn normalize_provider_slug(value: Option<&str>) -> Result<String, StatusCode> {
    let normalized = value.unwrap_or(PROVIDER_GROK).trim().to_ascii_lowercase();
    match normalized.as_str() {
        PROVIDER_GROK => Ok(PROVIDER_GROK.to_string()),
        PROVIDER_CODEX => Ok(PROVIDER_CODEX.to_string()),
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

fn normalize_account_credential(
    provider_slug: &str,
    value: &Value,
) -> Result<AccountCredential, StatusCode> {
    AccountCredential::from_provider_value(provider_slug, value).map_err(|error| {
        tracing::warn!(provider = provider_slug, %error, "Invalid account credential payload");
        StatusCode::BAD_REQUEST
    })
}

fn normalize_existing_credential(
    account: &accounts::DbAccount,
) -> Result<AccountCredential, String> {
    if let Some(payload) = account.credential_payload.as_ref() {
        return AccountCredential::from_provider_value(&account.provider_slug, payload);
    }

    if account.provider_slug == PROVIDER_GROK {
        return AccountCredential::from_provider_value(PROVIDER_GROK, &account.cookies);
    }

    Err(format!(
        "Account {} does not contain provider credentials",
        account.name
    ))
}

fn credential_preview_for_row(account: &accounts::DbAccount) -> Value {
    normalize_existing_credential(account)
        .map(|credential| credential.to_preview())
        .unwrap_or_else(|_| {
            json!({
                "type": if account.provider_slug == PROVIDER_CODEX {
                    CREDENTIAL_TYPE_CODEX_OAUTH_TOKENS
                } else {
                    CREDENTIAL_TYPE_GROK_COOKIES
                },
                "configured": false
            })
        })
}

async fn persist_account_credential(
    state: &AppState,
    account_id: i32,
    provider_slug: &str,
    credential: &AccountCredential,
) -> Result<(), String> {
    let payload = credential.to_provider_payload_value()?;
    account_credentials::upsert_account_credential(
        &state.db,
        account_id,
        credential.credential_type(),
        &payload,
    )
    .await
    .map_err(|error| format!("Persist account credential failed: {error}"))?;

    match credential {
        AccountCredential::GrokCookies(_) => {
            accounts::update_account_identity(
                &state.db,
                account_id,
                None,
                None,
                Some(AUTH_MODE_GROK_COOKIES),
                Some(&json!({ "provider": provider_slug })),
            )
            .await
            .map_err(|error| format!("Persist Grok account metadata failed: {error}"))?;
            sqlx::query(
                r#"
                UPDATE accounts
                SET
                    active = true,
                    session_status = 'healthy',
                    session_error = NULL,
                    session_checked_at = NOW(),
                    routing_state = 'healthy',
                    cooldown_until = NULL,
                    last_routing_error = NULL,
                    rate_limit_streak = 0,
                    auth_failure_streak = 0,
                    refresh_failure_streak = 0
                WHERE id = $1
                "#,
            )
            .bind(account_id)
            .execute(&state.db)
            .await
            .map_err(|error| format!("Update Grok account session state failed: {error}"))?;
        }
        AccountCredential::CodexTokens(tokens) => {
            codex_oauth::persist_tokens(&state.db, account_id, tokens).await?;
        }
    }

    Ok(())
}

async fn mark_codex_needs_login(
    pool: &sqlx::PgPool,
    id: i32,
    message: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE accounts
        SET
            session_status = 'needs_login',
            session_error = $1,
            session_checked_at = NOW(),
            routing_state = 'candidate',
            cooldown_until = NULL,
            last_routing_error = NULL
        WHERE id = $2
        "#,
    )
    .bind(message)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}
