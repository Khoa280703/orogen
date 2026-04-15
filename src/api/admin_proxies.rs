use axum::Json;
use axum::extract::Path;
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::db::proxies;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyCreateRequest {
    pub url: String,
    #[serde(default)]
    pub label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyUpdateRequest {
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ProxyResponse {
    pub id: i32,
    pub url: String, // Masked URL
    pub label: Option<String>,
    pub active: bool,
    pub created_at: Option<String>,
    pub assigned_accounts: i32,
}

/// Validate proxy URL format: socks5h://user:pass@host:port
fn validate_proxy_url(url: &str) -> bool {
    if !url.starts_with("socks5h://") {
        return false;
    }
    // Must contain @ for credentials and : for port
    if !url.contains('@') || !url.contains(':') {
        return false;
    }
    // Basic length check
    url.len() <= 500
}

/// Mask proxy URL credentials for safe display: socks5h://user:***@host:port
fn mask_proxy_url(url: &str) -> String {
    if let Some(at_pos) = url.find('@') {
        if let Some(scheme_end) = url.find("://") {
            let protocol = &url[..scheme_end + 3];
            let user_part = &url[scheme_end + 3..at_pos];
            if let Some(last_colon) = user_part.rfind(':') {
                let user = &user_part[..last_colon];
                let host_part = &url[at_pos + 1..];
                return format!("{}{}:***@{}", protocol, user, host_part);
            }
        }
    }
    // Fallback: mask everything after protocol
    format!(
        "{}***",
        if let Some(end) = url.find("://") {
            &url[..end + 3]
        } else {
            ""
        }
    )
}

/// List all proxies (with masked URLs)
pub async fn list_proxies(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<Vec<ProxyResponse>>, axum::http::StatusCode> {
    let db = &state.db;
    let proxies_list = proxies::list_proxies(&db)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut response = Vec::new();
    for proxy in proxies_list {
        let has_assigned = proxies::has_assigned_accounts(&db, proxy.id)
            .await
            .unwrap_or(false);
        response.push(ProxyResponse {
            id: proxy.id,
            url: mask_proxy_url(&proxy.url), // Mask credentials
            label: proxy.label,
            active: proxy.active.unwrap_or(true),
            created_at: proxy.created_at.map(|d| d.to_rfc3339()),
            assigned_accounts: if has_assigned { 1 } else { 0 },
        });
    }

    Ok(Json(response))
}

/// Create a new proxy (with validation)
pub async fn create_proxy(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(req): Json<ProxyCreateRequest>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    // Validate proxy URL format
    if !validate_proxy_url(&req.url) {
        tracing::warn!(url = %req.url, "Invalid proxy URL format");
        return Err(axum::http::StatusCode::BAD_REQUEST);
    }

    let db = &state.db;
    let id = proxies::create_proxy(&db, &req.url, req.label.as_deref())
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "id": id })))
}

/// Update a proxy (with validation)
pub async fn update_proxy(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<ProxyUpdateRequest>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    // Validate proxy URL format if provided
    if let Some(ref url) = req.url {
        if !validate_proxy_url(url) {
            tracing::warn!(id, url, "Invalid proxy URL format");
            return Err(axum::http::StatusCode::BAD_REQUEST);
        }
    }

    let db = &state.db;
    proxies::update_proxy(
        &db,
        id,
        req.url.as_deref(),
        req.label.as_deref(),
        req.active,
    )
    .await
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "success": true })))
}

/// Delete a proxy
pub async fn delete_proxy(
    axum::extract::State(state): axum::extract::State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let db = &state.db;

    // Check if proxy has assigned accounts
    if proxies::has_assigned_accounts(&db, id)
        .await
        .unwrap_or(false)
    {
        return Err(axum::http::StatusCode::CONFLICT);
    }

    proxies::delete_proxy(&db, id)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "success": true })))
}
