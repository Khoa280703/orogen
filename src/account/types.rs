use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;

pub const PROVIDER_GROK: &str = "grok";
pub const PROVIDER_CODEX: &str = "codex";

pub const CREDENTIAL_TYPE_GROK_COOKIES: &str = "grok_cookies";
pub const CREDENTIAL_TYPE_CODEX_OAUTH_TOKENS: &str = "codex_oauth_tokens";

pub const AUTH_MODE_GROK_COOKIES: &str = "grok_cookies";
pub const AUTH_MODE_CODEX_OAUTH: &str = "codex_oauth";

pub const ROUTING_STATE_HEALTHY: &str = "healthy";
pub const ROUTING_STATE_COOLING_DOWN: &str = "cooling_down";
pub const ROUTING_STATE_AUTH_INVALID: &str = "auth_invalid";
pub const ROUTING_STATE_REFRESH_FAILED: &str = "refresh_failed";
pub const ROUTING_STATE_PAUSED: &str = "paused";
pub const ROUTING_STATE_CANDIDATE: &str = "candidate";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrokCookies {
    pub sso: String,
    #[serde(rename = "sso-rw", default)]
    pub sso_rw: Option<String>,
    #[serde(default)]
    pub cf_clearance: Option<String>,
    #[serde(rename = "_raw", default)]
    pub raw: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl GrokCookies {
    pub fn from_value(value: &Value) -> Result<Self, String> {
        match value {
            Value::String(raw) => Self::from_raw_cookie_header(raw),
            Value::Object(_) => {
                let cookies: Self = serde_json::from_value(value.clone())
                    .map_err(|error| format!("Invalid cookies JSON: {error}"))?;
                if cookies.sso.trim().is_empty() {
                    return Err("Missing sso cookie".into());
                }
                Ok(cookies)
            }
            _ => Err("Cookies must be either a JSON object or raw cookie string".into()),
        }
    }

    pub fn from_raw_cookie_header(raw: &str) -> Result<Self, String> {
        let raw = raw.trim();
        if raw.is_empty() {
            return Err("Cookie string is empty".into());
        }

        let mut sso = None;
        let mut sso_rw = None;
        let mut cf_clearance = None;
        let mut extra = HashMap::new();

        for part in raw.split(';') {
            let entry = part.trim();
            if entry.is_empty() {
                continue;
            }

            let Some((name, value)) = entry.split_once('=') else {
                continue;
            };

            let key = name.trim();
            let value = value.trim().to_string();
            match key {
                "sso" => sso = Some(value.clone()),
                "sso-rw" => sso_rw = Some(value.clone()),
                "cf_clearance" => cf_clearance = Some(value.clone()),
                _ => {}
            }
            extra.insert(key.to_string(), json!(value));
        }

        let Some(sso) = sso else {
            return Err("Missing sso cookie".into());
        };

        Ok(Self {
            sso,
            sso_rw,
            cf_clearance,
            raw: Some(raw.to_string()),
            extra,
        })
    }

    pub fn to_header(&self) -> String {
        if let Some(raw) = &self.raw {
            return raw.clone();
        }
        let mut parts = vec![format!("sso={}", self.sso)];
        if let Some(rw) = &self.sso_rw {
            parts.push(format!("sso-rw={rw}"));
        }
        if let Some(cf) = &self.cf_clearance {
            parts.push(format!("cf_clearance={cf}"));
        }
        parts.join("; ")
    }

    pub fn to_preview(&self) -> Value {
        json!({
            "type": CREDENTIAL_TYPE_GROK_COOKIES,
            "has_sso": !self.sso.trim().is_empty(),
            "has_sso_rw": self.sso_rw.as_ref().is_some_and(|value| !value.trim().is_empty()),
            "has_cf_clearance": self.cf_clearance.as_ref().is_some_and(|value| !value.trim().is_empty()),
            "raw_cookie_present": self.raw.as_ref().is_some_and(|value| !value.trim().is_empty()),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodexTokens {
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub id_token: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub last_refresh_at: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub token_type: Option<String>,
}

impl CodexTokens {
    pub fn from_value(value: &Value) -> Result<Self, String> {
        let tokens: Self = serde_json::from_value(value.clone())
            .map_err(|error| format!("Invalid Codex token payload: {error}"))?;
        if tokens.access_token.trim().is_empty() {
            return Err("Missing access_token".into());
        }
        Ok(tokens)
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at
            .as_deref()
            .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
            .map(|value| value.with_timezone(&chrono::Utc) <= chrono::Utc::now())
            .unwrap_or(false)
    }

    pub fn should_refresh(&self, lead_time_seconds: i64) -> bool {
        self.expires_at
            .as_deref()
            .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
            .map(|value| {
                value.with_timezone(&chrono::Utc)
                    <= chrono::Utc::now() + chrono::Duration::seconds(lead_time_seconds)
            })
            .unwrap_or(false)
    }

    pub fn to_preview(&self) -> Value {
        json!({
            "type": CREDENTIAL_TYPE_CODEX_OAUTH_TOKENS,
            "email": self.email,
            "account_id": self.account_id,
            "expires_at": self.expires_at,
            "last_refresh_at": self.last_refresh_at,
            "has_refresh_token": self.refresh_token.as_ref().is_some_and(|value| !value.trim().is_empty()),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider", content = "payload", rename_all = "snake_case")]
pub enum AccountCredential {
    GrokCookies(GrokCookies),
    CodexTokens(CodexTokens),
}

impl AccountCredential {
    pub fn provider_slug(&self) -> &'static str {
        match self {
            Self::GrokCookies(_) => PROVIDER_GROK,
            Self::CodexTokens(_) => PROVIDER_CODEX,
        }
    }

    pub fn credential_type(&self) -> &'static str {
        match self {
            Self::GrokCookies(_) => CREDENTIAL_TYPE_GROK_COOKIES,
            Self::CodexTokens(_) => CREDENTIAL_TYPE_CODEX_OAUTH_TOKENS,
        }
    }

    pub fn from_provider_value(provider_slug: &str, value: &Value) -> Result<Self, String> {
        match provider_slug {
            PROVIDER_GROK => Ok(Self::GrokCookies(GrokCookies::from_value(value)?)),
            PROVIDER_CODEX => Ok(Self::CodexTokens(CodexTokens::from_value(value)?)),
            other => Err(format!("Unsupported provider credential type: {other}")),
        }
    }

    pub fn to_payload_value(&self) -> Result<Value, String> {
        serde_json::to_value(self).map_err(|error| format!("Serialize credential failed: {error}"))
    }

    pub fn to_provider_payload_value(&self) -> Result<Value, String> {
        match self {
            Self::GrokCookies(cookies) => serde_json::to_value(cookies)
                .map_err(|error| format!("Serialize grok cookies failed: {error}")),
            Self::CodexTokens(tokens) => serde_json::to_value(tokens)
                .map_err(|error| format!("Serialize codex tokens failed: {error}")),
        }
    }

    pub fn to_preview(&self) -> Value {
        match self {
            Self::GrokCookies(cookies) => cookies.to_preview(),
            Self::CodexTokens(tokens) => tokens.to_preview(),
        }
    }

    pub fn as_grok_cookies(&self) -> Option<&GrokCookies> {
        match self {
            Self::GrokCookies(cookies) => Some(cookies),
            _ => None,
        }
    }

    pub fn as_codex_tokens(&self) -> Option<&CodexTokens> {
        match self {
            Self::CodexTokens(tokens) => Some(tokens),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountEntry {
    pub name: String,
    pub provider_slug: String,
    pub credential_preview: Value,
    #[serde(default)]
    pub account_label: Option<String>,
    #[serde(default)]
    pub external_account_id: Option<String>,
    #[serde(default = "default_true")]
    pub active: bool,
    #[serde(default)]
    pub request_count: u64,
    #[serde(default)]
    pub last_used: Option<String>,
    #[serde(default, skip_serializing)]
    pub proxy_url: Option<String>,
    #[serde(default, skip_serializing)]
    pub fail_count: u32,
    #[serde(default, skip_serializing)]
    pub success_count: u64,
}

fn default_true() -> bool {
    true
}
