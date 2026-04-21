use std::path::PathBuf;

use serde::Deserialize;
use tokio::process::Command;

use crate::account::types::GrokCookies;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ScriptResponse {
    ok: bool,
    error: Option<String>,
    message: Option<String>,
    profile_dir: Option<String>,
    pid: Option<u32>,
    cookies: Option<serde_json::Value>,
}

#[derive(Debug)]
pub struct LaunchLoginBrowserResult {
    pub profile_dir: String,
    pub pid: Option<u32>,
    pub message: Option<String>,
}

#[derive(Debug)]
pub struct SyncProfileCookiesResult {
    pub profile_dir: String,
    pub cookies: GrokCookies,
    pub message: Option<String>,
}

pub fn resolve_profile_dir(account_name: &str) -> Result<PathBuf, String> {
    let base =
        std::env::current_dir().map_err(|error| format!("Cannot read current dir: {error}"))?;
    Ok(base
        .join("data")
        .join("browser-profiles")
        .join(sanitize_account_name(account_name)))
}

pub async fn launch_login_browser(account_name: &str) -> Result<LaunchLoginBrowserResult, String> {
    let profile_dir = resolve_profile_dir(account_name)?;
    let response = run_script(&["launch-login", profile_dir.to_string_lossy().as_ref()]).await?;

    Ok(LaunchLoginBrowserResult {
        profile_dir: response
            .profile_dir
            .unwrap_or_else(|| profile_dir.to_string_lossy().to_string()),
        pid: response.pid,
        message: response.message,
    })
}

pub async fn sync_profile_cookies(account_name: &str) -> Result<SyncProfileCookiesResult, String> {
    let profile_dir = resolve_profile_dir(account_name)?;
    let response = run_script(&["sync-cookies", profile_dir.to_string_lossy().as_ref()]).await?;
    let cookies_value = response
        .cookies
        .ok_or_else(|| "Profile sync did not return cookies".to_string())?;
    let cookies = GrokCookies::from_value(&cookies_value)?;

    Ok(SyncProfileCookiesResult {
        profile_dir: response
            .profile_dir
            .unwrap_or_else(|| profile_dir.to_string_lossy().to_string()),
        cookies,
        message: response.message,
    })
}

async fn run_script(args: &[&str]) -> Result<ScriptResponse, String> {
    let root =
        std::env::current_dir().map_err(|error| format!("Cannot read current dir: {error}"))?;
    let script_path = root.join("scripts").join("grok-profile-session.mjs");

    if !script_path.exists() {
        return Err(format!(
            "Missing profile session script: {}",
            script_path.display()
        ));
    }

    let output = Command::new("node")
        .arg(&script_path)
        .args(args)
        .output()
        .await
        .map_err(|error| format!("Failed to run node script: {error}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    let response = serde_json::from_str::<ScriptResponse>(&stdout).map_err(|error| {
        if stderr.is_empty() {
            format!("Invalid script response: {error}")
        } else {
            format!("Invalid script response: {error}. stderr: {stderr}")
        }
    })?;

    if output.status.success() && response.ok {
        return Ok(response);
    }

    Err(response.error.unwrap_or_else(|| {
        if stderr.is_empty() {
            "Profile session command failed".to_string()
        } else {
            stderr
        }
    }))
}

fn sanitize_account_name(account_name: &str) -> String {
    let sanitized = account_name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>();

    if sanitized.is_empty() {
        "account".to_string()
    } else {
        sanitized
    }
}
