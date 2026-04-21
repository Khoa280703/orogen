use serde::{Deserialize, Serialize, Serializer, de::Deserializer};
use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::middleware::csrf::CsrfProtection;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppConfigInner {
    #[serde(default = "default_port")]
    pub api_port: u16,
    #[serde(default, rename = "apiToken")]
    pub api_token: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub api_keys: Vec<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "adminToken"
    )]
    pub admin_token: Option<String>,
    #[serde(default = "default_model")]
    pub default_model: String,
    #[serde(default = "default_data_dir")]
    pub data_dir: String,
    #[serde(skip)]
    pub database_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_oauth_client_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_oauth_client_secret: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_oauth_auth_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_oauth_token_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_oauth_redirect_url: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub codex_oauth_scopes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_upstream_base_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_upstream_originator: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_upstream_user_agent: Option<String>,
}

pub struct AppConfig {
    pub api_port: u16,
    pub api_token: String,
    pub api_keys: Vec<String>,
    pub admin_token: Option<String>,
    pub default_model: String,
    pub data_dir: String,
    pub database_url: Option<String>,
    pub codex_oauth_client_id: Option<String>,
    pub codex_oauth_client_secret: Option<String>,
    pub codex_oauth_auth_url: String,
    pub codex_oauth_token_url: String,
    pub codex_oauth_redirect_url: Option<String>,
    pub codex_oauth_scopes: Vec<String>,
    pub codex_upstream_base_url: String,
    pub codex_upstream_originator: String,
    pub codex_upstream_user_agent: String,
    pub csrf_protection: Arc<CsrfProtection>,
}

impl Serialize for AppConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let inner = AppConfigInner {
            api_port: self.api_port,
            api_token: self.api_token.clone(),
            api_keys: self.api_keys.clone(),
            admin_token: self.admin_token.clone(),
            default_model: self.default_model.clone(),
            data_dir: self.data_dir.clone(),
            database_url: None,
            codex_oauth_client_id: self.codex_oauth_client_id.clone(),
            codex_oauth_client_secret: None,
            codex_oauth_auth_url: Some(self.codex_oauth_auth_url.clone()),
            codex_oauth_token_url: Some(self.codex_oauth_token_url.clone()),
            codex_oauth_redirect_url: self.codex_oauth_redirect_url.clone(),
            codex_oauth_scopes: self.codex_oauth_scopes.clone(),
            codex_upstream_base_url: Some(self.codex_upstream_base_url.clone()),
            codex_upstream_originator: Some(self.codex_upstream_originator.clone()),
            codex_upstream_user_agent: Some(self.codex_upstream_user_agent.clone()),
        };
        inner.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AppConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = AppConfigInner::deserialize(deserializer)?;
        Ok(AppConfig {
            api_port: inner.api_port,
            api_token: inner.api_token,
            api_keys: inner.api_keys,
            admin_token: inner.admin_token,
            default_model: inner.default_model,
            data_dir: inner.data_dir,
            database_url: inner.database_url,
            codex_oauth_client_id: inner.codex_oauth_client_id,
            codex_oauth_client_secret: inner.codex_oauth_client_secret,
            codex_oauth_auth_url: inner
                .codex_oauth_auth_url
                .unwrap_or_else(default_codex_oauth_auth_url),
            codex_oauth_token_url: inner
                .codex_oauth_token_url
                .unwrap_or_else(default_codex_oauth_token_url),
            codex_oauth_redirect_url: inner.codex_oauth_redirect_url,
            codex_oauth_scopes: if inner.codex_oauth_scopes.is_empty() {
                default_codex_oauth_scopes()
            } else {
                inner.codex_oauth_scopes
            },
            codex_upstream_base_url: inner
                .codex_upstream_base_url
                .unwrap_or_else(default_codex_upstream_base_url),
            codex_upstream_originator: inner
                .codex_upstream_originator
                .unwrap_or_else(default_codex_upstream_originator),
            codex_upstream_user_agent: inner
                .codex_upstream_user_agent
                .unwrap_or_else(default_codex_upstream_user_agent),
            csrf_protection: Arc::new(CsrfProtection::new()),
        })
    }
}

fn default_port() -> u16 {
    3069
}
fn default_model() -> String {
    "grok-3".into()
}
fn default_data_dir() -> String {
    "data/conversations".into()
}
fn default_codex_oauth_auth_url() -> String {
    "https://auth.openai.com/authorize".into()
}
fn default_codex_oauth_token_url() -> String {
    "https://auth.openai.com/oauth/token".into()
}
fn default_codex_oauth_scopes() -> Vec<String> {
    vec!["openid".into(), "profile".into(), "offline_access".into()]
}
fn default_codex_upstream_base_url() -> String {
    "https://chatgpt.com/backend-api/codex/responses".into()
}
fn default_codex_upstream_originator() -> String {
    "codex-cli".into()
}
fn default_codex_upstream_user_agent() -> String {
    format!(
        "codex-cli/1.0.18 ({}; {})",
        std::env::consts::OS,
        std::env::consts::ARCH
    )
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api_port: default_port(),
            api_token: String::new(),
            api_keys: Vec::new(),
            admin_token: None,
            default_model: default_model(),
            data_dir: default_data_dir(),
            database_url: None,
            codex_oauth_client_id: None,
            codex_oauth_client_secret: None,
            codex_oauth_auth_url: default_codex_oauth_auth_url(),
            codex_oauth_token_url: default_codex_oauth_token_url(),
            codex_oauth_redirect_url: None,
            codex_oauth_scopes: default_codex_oauth_scopes(),
            codex_upstream_base_url: default_codex_upstream_base_url(),
            codex_upstream_originator: default_codex_upstream_originator(),
            codex_upstream_user_agent: default_codex_upstream_user_agent(),
            csrf_protection: Arc::new(CsrfProtection::new()),
        }
    }
}

impl Clone for AppConfig {
    fn clone(&self) -> Self {
        Self {
            api_port: self.api_port,
            api_token: self.api_token.clone(),
            api_keys: self.api_keys.clone(),
            admin_token: self.admin_token.clone(),
            default_model: self.default_model.clone(),
            data_dir: self.data_dir.clone(),
            database_url: self.database_url.clone(),
            codex_oauth_client_id: self.codex_oauth_client_id.clone(),
            codex_oauth_client_secret: self.codex_oauth_client_secret.clone(),
            codex_oauth_auth_url: self.codex_oauth_auth_url.clone(),
            codex_oauth_token_url: self.codex_oauth_token_url.clone(),
            codex_oauth_redirect_url: self.codex_oauth_redirect_url.clone(),
            codex_oauth_scopes: self.codex_oauth_scopes.clone(),
            codex_upstream_base_url: self.codex_upstream_base_url.clone(),
            codex_upstream_originator: self.codex_upstream_originator.clone(),
            codex_upstream_user_agent: self.codex_upstream_user_agent.clone(),
            csrf_protection: self.csrf_protection.clone(),
        }
    }
}

pub fn load_config() -> AppConfig {
    let path = Path::new("config.json");
    let mut config = if path.exists() {
        let raw = fs::read_to_string(path).unwrap_or_default();
        serde_json::from_str(&raw).unwrap_or_default()
    } else {
        let config = AppConfig::default();
        let _ = fs::write(path, serde_json::to_string_pretty(&config).unwrap());
        config
    };

    // Load DATABASE_URL from env var
    if let Ok(url) = std::env::var("DATABASE_URL") {
        config.database_url = Some(url);
    }

    // Load admin_token from env var if not in config
    if config.admin_token.is_none() {
        if let Ok(token) = std::env::var("ADMIN_TOKEN") {
            config.admin_token = Some(token);
        }
    }

    if let Ok(value) = std::env::var("CODEX_OAUTH_CLIENT_ID") {
        config.codex_oauth_client_id = Some(value);
    }

    if let Ok(value) = std::env::var("CODEX_OAUTH_CLIENT_SECRET") {
        config.codex_oauth_client_secret = Some(value);
    }

    if let Ok(value) = std::env::var("CODEX_OAUTH_AUTH_URL") {
        config.codex_oauth_auth_url = value;
    }

    if let Ok(value) = std::env::var("CODEX_OAUTH_TOKEN_URL") {
        config.codex_oauth_token_url = value;
    }

    if let Ok(value) = std::env::var("CODEX_OAUTH_REDIRECT_URL") {
        config.codex_oauth_redirect_url = Some(value);
    }

    if let Ok(value) = std::env::var("CODEX_OAUTH_SCOPES") {
        config.codex_oauth_scopes = value
            .split(',')
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(str::to_string)
            .collect();
    }

    if let Ok(value) = std::env::var("CODEX_UPSTREAM_BASE_URL") {
        config.codex_upstream_base_url = value;
    }

    if let Ok(value) = std::env::var("CODEX_UPSTREAM_ORIGINATOR") {
        config.codex_upstream_originator = value;
    }

    if let Ok(value) = std::env::var("CODEX_UPSTREAM_USER_AGENT") {
        config.codex_upstream_user_agent = value;
    }

    config
}

impl AppConfig {
    /// Get all API keys merged from `api_token` and `api_keys` fields
    pub fn all_keys(&self) -> std::collections::HashSet<String> {
        let mut keys = std::collections::HashSet::new();
        if !self.api_token.is_empty() {
            keys.insert(self.api_token.clone());
        }
        for key in &self.api_keys {
            keys.insert(key.clone());
        }
        keys
    }
}
