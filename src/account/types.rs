use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrokCookies {
    pub sso: String,
    #[serde(rename = "sso-rw", default)]
    pub sso_rw: Option<String>,
    #[serde(default)]
    pub cf_clearance: Option<String>,
    /// Full raw cookie string for direct header injection
    #[serde(rename = "_raw", default)]
    pub raw: Option<String>,
    /// Capture any extra cookie fields
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

    /// Build cookie header string
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountEntry {
    pub name: String,
    pub cookies: GrokCookies,
    #[serde(default = "default_true")]
    pub active: bool,
    #[serde(default)]
    pub request_count: u64,
    #[serde(default)]
    pub last_used: Option<String>,
    /// Bound proxy URL for this account (not persisted to cookies.json)
    #[serde(default, skip_serializing)]
    pub proxy_url: Option<String>,
    /// Consecutive failure count for health monitoring (runtime only)
    #[serde(default, skip_serializing)]
    pub fail_count: u32,
    /// Total success count for health monitoring (runtime only)
    #[serde(default, skip_serializing)]
    pub success_count: u64,
}

fn default_true() -> bool {
    true
}
