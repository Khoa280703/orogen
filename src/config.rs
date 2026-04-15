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
}

pub struct AppConfig {
    pub api_port: u16,
    pub api_token: String,
    pub api_keys: Vec<String>,
    pub admin_token: Option<String>,
    pub default_model: String,
    pub data_dir: String,
    pub database_url: Option<String>,
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
