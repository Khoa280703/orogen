use axum::Json;
use axum::extract::Path;
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::db::proxies;

const ALLOWED_PROXY_SCHEMES: &[&str] = &["http", "https", "socks5", "socks5h"];

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

/// Validate proxy URL format: http(s)://host:port or socks5(h)://user:pass@host:port
fn validate_proxy_url(url: &str) -> bool {
    let trimmed = url.trim();
    if trimmed.is_empty() || trimmed.len() > 500 {
        return false;
    }

    let Some((scheme, remainder)) = trimmed.split_once("://") else {
        return false;
    };

    if !ALLOWED_PROXY_SCHEMES.contains(&scheme) {
        return false;
    }

    let authority = remainder.split('/').next().unwrap_or_default();
    if authority.is_empty() {
        return false;
    }

    let host_port = authority
        .rsplit_once('@')
        .map(|(_, host_port)| host_port)
        .unwrap_or(authority);

    let Some((host, port)) = host_port.rsplit_once(':') else {
        return false;
    };

    !host.is_empty() && port.parse::<u16>().is_ok()
}

/// Mask proxy URL credentials for safe display while leaving auth-less proxies unchanged.
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
    url.to_string()
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

#[cfg(test)]
mod tests {
    use super::{mask_proxy_url, validate_proxy_url};

    #[test]
    fn accepts_http_and_socks_proxy_urls() {
        assert!(validate_proxy_url("http://38.154.150.171:8800"));
        assert!(validate_proxy_url("https://38.154.150.171:8800"));
        assert!(validate_proxy_url("socks5://user:pass@38.154.150.171:8800"));
        assert!(validate_proxy_url(
            "socks5h://user:pass@38.154.150.171:8800"
        ));
    }

    #[test]
    fn rejects_invalid_proxy_urls() {
        assert!(!validate_proxy_url("38.154.150.171:8800"));
        assert!(!validate_proxy_url("ftp://38.154.150.171:8800"));
        assert!(!validate_proxy_url("http://38.154.150.171"));
        assert!(!validate_proxy_url("http://:8800"));
    }

    #[test]
    fn masks_only_urls_with_credentials() {
        assert_eq!(
            mask_proxy_url("socks5h://user:pass@38.154.150.171:8800"),
            "socks5h://user:***@38.154.150.171:8800"
        );
        assert_eq!(
            mask_proxy_url("http://38.154.150.171:8800"),
            "http://38.154.150.171:8800"
        );
    }
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
