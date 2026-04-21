use std::collections::BTreeMap;

use chrono::Utc;
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};

use crate::account::types::{AccountCredential, PROVIDER_CODEX};
use crate::config::AppConfig;
use crate::db::{accounts, proxies};
use crate::services::codex_oauth;

const CODEX_USAGE_URL: &str = "https://chatgpt.com/backend-api/wham/usage";

#[derive(Debug, Clone, Serialize)]
pub struct ProviderAccountUsage {
    pub account_id: i32,
    pub provider_slug: String,
    pub supported: bool,
    pub fetched_at: String,
    pub plan: Option<String>,
    pub limit_reached: Option<bool>,
    pub message: Option<String>,
    pub quotas: BTreeMap<String, ProviderAccountQuota>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderAccountQuota {
    pub used: i32,
    pub total: i32,
    pub remaining: i32,
    pub remaining_percentage: i32,
    pub reset_at: Option<String>,
    pub unlimited: bool,
}

#[derive(Debug, Default, Deserialize)]
struct CodexUsagePayload {
    #[serde(default)]
    plan_type: Option<String>,
    #[serde(default)]
    rate_limit: CodexRateLimit,
}

#[derive(Debug, Default, Deserialize)]
struct CodexRateLimit {
    #[serde(default)]
    limit_reached: bool,
    #[serde(default)]
    primary_window: CodexRateWindow,
    #[serde(default)]
    secondary_window: CodexRateWindow,
}

#[derive(Debug, Default, Deserialize)]
struct CodexRateWindow {
    #[serde(default)]
    used_percent: i32,
    #[serde(default)]
    reset_at: Option<i64>,
}

pub async fn fetch_account_usage(
    db: &sqlx::PgPool,
    config: &AppConfig,
    account: &accounts::DbAccount,
) -> Result<ProviderAccountUsage, String> {
    let fetched_at = Utc::now().to_rfc3339();
    let proxy_url = resolve_proxy_url(db, account.proxy_id).await?;
    let credential = normalize_existing_credential(account)?;

    match account.provider_slug.as_str() {
        PROVIDER_CODEX => {
            fetch_codex_usage(db, config, account.id, credential, proxy_url, fetched_at).await
        }
        _ => Ok(ProviderAccountUsage {
            account_id: account.id,
            provider_slug: account.provider_slug.clone(),
            supported: false,
            fetched_at,
            plan: None,
            limit_reached: None,
            message: Some(format!(
                "Usage API is not implemented yet for provider {}.",
                account.provider_slug
            )),
            quotas: BTreeMap::new(),
        }),
    }
}

async fn fetch_codex_usage(
    db: &sqlx::PgPool,
    config: &AppConfig,
    account_id: i32,
    credential: AccountCredential,
    proxy_url: Option<String>,
    fetched_at: String,
) -> Result<ProviderAccountUsage, String> {
    let mut tokens = credential
        .as_codex_tokens()
        .cloned()
        .ok_or_else(|| "Codex account tokens not found".to_string())?;

    if tokens.should_refresh(120) || tokens.is_expired() {
        if let Ok(refreshed) =
            codex_oauth::refresh_account_tokens(db, config, account_id, &tokens).await
        {
            tokens = refreshed;
        }
    }

    let client = build_http_client(proxy_url.as_deref())?;
    let response = client
        .get(CODEX_USAGE_URL)
        .header(AUTHORIZATION, format!("Bearer {}", tokens.access_token))
        .header(ACCEPT, "application/json")
        .header(USER_AGENT, config.codex_upstream_user_agent.as_str())
        .send()
        .await
        .map_err(|error| format!("Codex usage request failed: {error}"))?;

    if !response.status().is_success() {
        return Ok(ProviderAccountUsage {
            account_id,
            provider_slug: PROVIDER_CODEX.to_string(),
            supported: true,
            fetched_at,
            plan: None,
            limit_reached: None,
            message: Some(format!(
                "Codex connected. Usage API temporarily unavailable ({}).",
                response.status()
            )),
            quotas: BTreeMap::new(),
        });
    }

    let payload: CodexUsagePayload = response
        .json()
        .await
        .map_err(|error| format!("Decode Codex usage payload failed: {error}"))?;

    let mut quotas = BTreeMap::new();
    quotas.insert(
        "session".to_string(),
        quota_from_window(&payload.rate_limit.primary_window),
    );
    quotas.insert(
        "weekly".to_string(),
        quota_from_window(&payload.rate_limit.secondary_window),
    );

    Ok(ProviderAccountUsage {
        account_id,
        provider_slug: PROVIDER_CODEX.to_string(),
        supported: true,
        fetched_at,
        plan: payload.plan_type,
        limit_reached: Some(payload.rate_limit.limit_reached),
        message: None,
        quotas,
    })
}

fn quota_from_window(window: &CodexRateWindow) -> ProviderAccountQuota {
    let used = window.used_percent.clamp(0, 100);
    ProviderAccountQuota {
        used,
        total: 100,
        remaining: 100 - used,
        remaining_percentage: 100 - used,
        reset_at: window.reset_at.and_then(|value| {
            chrono::DateTime::from_timestamp(value, 0).map(|date| date.to_rfc3339())
        }),
        unlimited: false,
    }
}

fn build_http_client(proxy_url: Option<&str>) -> Result<reqwest::Client, String> {
    let mut builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .connect_timeout(std::time::Duration::from_secs(10));

    if let Some(url) = proxy_url.filter(|value| !value.trim().is_empty()) {
        builder = builder.proxy(
            reqwest::Proxy::all(url).map_err(|error| format!("Invalid proxy URL: {error}"))?,
        );
    }

    builder
        .build()
        .map_err(|error| format!("Build usage HTTP client failed: {error}"))
}

fn normalize_existing_credential(
    account: &accounts::DbAccount,
) -> Result<AccountCredential, String> {
    if let Some(payload) = account.credential_payload.as_ref() {
        return AccountCredential::from_provider_value(&account.provider_slug, payload);
    }

    AccountCredential::from_provider_value(&account.provider_slug, &account.cookies)
}

async fn resolve_proxy_url(
    db: &sqlx::PgPool,
    proxy_id: Option<i32>,
) -> Result<Option<String>, String> {
    let Some(id) = proxy_id else {
        return Ok(None);
    };

    let proxy = proxies::get_proxy(db, id)
        .await
        .map_err(|error| format!("Load proxy failed: {error}"))?;

    Ok(proxy.and_then(|value| value.active.unwrap_or(true).then_some(value.url)))
}
