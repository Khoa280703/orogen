use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{Duration, TimeZone, Utc};
use rand::Rng;
use rand::distr::Alphanumeric;
use reqwest::Url;
use reqwest::redirect::Policy;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::{RwLock, mpsc};

use crate::account::types::{
    AUTH_MODE_CODEX_OAUTH, CREDENTIAL_TYPE_CODEX_OAUTH_TOKENS, CodexTokens,
};
use crate::config::AppConfig;
use crate::db::{account_credentials, accounts};

const DEFAULT_CODEX_OAUTH_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const CODEX_LOGIN_EXPIRY_MINUTES: i64 = 15;
const CODEX_LOGIN_STALE_MINUTES: i64 = 20;

pub type CodexLoginSessionStore = Arc<RwLock<HashMap<String, CodexLoginSession>>>;

#[derive(Debug, Clone)]
pub struct CodexLoginSession {
    pub session_id: String,
    pub account_id: i32,
    pub status: String,
    pub verification_url: String,
    pub user_code: Option<String>,
    pub expires_at: Option<chrono::DateTime<Utc>>,
    pub command: String,
    pub message: Option<String>,
    pub started_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CodexLoginSessionView {
    pub session_id: String,
    pub account_id: i32,
    pub status: String,
    pub verification_url: String,
    pub user_code: Option<String>,
    pub expires_at: Option<String>,
    pub command: String,
    pub message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CodexTokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    id_token: Option<String>,
    #[serde(default)]
    expires_in: Option<i64>,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    token_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NativeCodexAuthFile {
    #[serde(default)]
    last_refresh: Option<String>,
    tokens: NativeCodexAuthTokens,
}

#[derive(Debug, Deserialize)]
struct NativeCodexAuthTokens {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    id_token: Option<String>,
    #[serde(default)]
    account_id: Option<String>,
}

pub async fn start_device_login(
    db: &sqlx::PgPool,
    config: &AppConfig,
    sessions: &CodexLoginSessionStore,
    account_id: i32,
) -> Result<CodexLoginSessionView, String> {
    cleanup_stale_login_sessions(sessions).await;

    if let Some(existing) = current_login_session_for_account(sessions, account_id).await {
        if !is_terminal_login_status(&existing.status) {
            return Ok(existing.to_view());
        }
    }

    let home_dir = codex_account_home_dir(config, account_id);
    std::fs::create_dir_all(home_dir.join(".codex"))
        .map_err(|error| format!("Create Codex account home failed: {error}"))?;

    let session = CodexLoginSession {
        session_id: random_token(24),
        account_id,
        status: "starting".to_string(),
        verification_url: String::new(),
        user_code: None,
        expires_at: None,
        command: format!("HOME={} codex login", home_dir.to_string_lossy()),
        message: Some("Starting native Codex browser login session...".to_string()),
        started_at: Utc::now(),
    };

    let session_id = session.session_id.clone();
    sessions
        .write()
        .await
        .insert(session.session_id.clone(), session.clone());

    tokio::spawn(monitor_device_login(
        db.clone(),
        config.clone(),
        sessions.clone(),
        session_id,
        account_id,
        home_dir,
    ));

    for _ in 0..30 {
        if let Some(current) = get_login_session(sessions, &session.session_id).await {
            if current.status != "starting" || is_terminal_login_status(&current.status) {
                return Ok(current.to_view());
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    get_login_session(sessions, &session.session_id)
        .await
        .map(|session| session.to_view())
        .ok_or_else(|| "Codex login session disappeared unexpectedly".to_string())
}

pub async fn get_login_status_for_account(
    sessions: &CodexLoginSessionStore,
    account_id: i32,
) -> Option<CodexLoginSessionView> {
    cleanup_stale_login_sessions(sessions).await;
    current_login_session_for_account(sessions, account_id)
        .await
        .map(|session| session.to_view())
}

pub async fn submit_manual_callback_url(
    sessions: &CodexLoginSessionStore,
    account_id: i32,
    callback_url: &str,
) -> Result<CodexLoginSessionView, String> {
    cleanup_stale_login_sessions(sessions).await;

    let session = current_login_session_for_account(sessions, account_id)
        .await
        .ok_or_else(|| "No active Codex login session for this account".to_string())?;

    if is_terminal_login_status(&session.status) {
        return Ok(session.to_view());
    }

    let callback = validate_manual_callback_url(callback_url, &session)?;
    let client = reqwest::Client::builder()
        .redirect(Policy::limited(10))
        .build()
        .map_err(|error| format!("Create local callback client failed: {error}"))?;

    client
        .get(callback)
        .send()
        .await
        .map_err(|error| format!("Submit local Codex callback failed: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Local Codex callback was rejected: {error}"))?;

    for _ in 0..30 {
        if let Some(current) = get_login_session(sessions, &session.session_id).await {
            if current.status != "awaiting_user" {
                return Ok(current.to_view());
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }

    get_login_session(sessions, &session.session_id)
        .await
        .map(|current| current.to_view())
        .ok_or_else(|| "Codex login session disappeared unexpectedly".to_string())
}

pub async fn refresh_account_tokens(
    db: &sqlx::PgPool,
    config: &AppConfig,
    account_id: i32,
    tokens: &CodexTokens,
) -> Result<CodexTokens, String> {
    let refresh_token = tokens
        .refresh_token
        .as_deref()
        .ok_or_else(|| "Codex account does not contain refresh_token".to_string())?;

    let client = reqwest::Client::new();
    let mut form: HashMap<&str, String> = HashMap::from([
        ("grant_type", "refresh_token".to_string()),
        ("refresh_token", refresh_token.to_string()),
        ("client_id", oauth_client_id(config)),
    ]);
    if let Some(secret) = config.codex_oauth_client_secret.as_deref() {
        form.insert("client_secret", secret.to_string());
    }

    let response = client
        .post(&config.codex_oauth_token_url)
        .form(&form)
        .send()
        .await
        .map_err(|error| format!("Codex token refresh failed: {error}"))?;

    let refreshed = merge_refreshed_tokens(tokens, parse_token_response(response).await?);
    persist_tokens(db, account_id, &refreshed).await?;
    Ok(refreshed)
}

pub async fn persist_tokens(
    db: &sqlx::PgPool,
    account_id: i32,
    tokens: &CodexTokens,
) -> Result<(), String> {
    let payload = serde_json::to_value(tokens)
        .map_err(|error| format!("Serialize Codex tokens failed: {error}"))?;
    account_credentials::upsert_account_credential(
        db,
        account_id,
        CREDENTIAL_TYPE_CODEX_OAUTH_TOKENS,
        &payload,
    )
    .await
    .map_err(|error| format!("Persist Codex credentials failed: {error}"))?;

    let metadata = json!({
        "email": tokens.email,
        "expires_at": tokens.expires_at,
        "last_refresh_at": tokens.last_refresh_at,
    });
    accounts::update_account_identity(
        db,
        account_id,
        tokens.email.as_deref(),
        tokens.account_id.as_deref(),
        Some(AUTH_MODE_CODEX_OAUTH),
        Some(&metadata),
    )
    .await
    .map_err(|error| format!("Persist Codex account metadata failed: {error}"))?;

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
            last_routing_error = NULL
        WHERE id = $1
        "#,
    )
    .bind(account_id)
    .execute(db)
    .await
    .map_err(|error| format!("Update Codex account session state failed: {error}"))?;

    Ok(())
}

pub async fn mark_refresh_failed(
    db: &sqlx::PgPool,
    account_id: i32,
    message: &str,
) -> Result<(), String> {
    accounts::mark_account_refresh_failed(db, account_id, message)
        .await
        .map_err(|error| format!("Update Codex routing refresh failure state failed: {error}"))?;

    sqlx::query(
        r#"
        UPDATE accounts
        SET
            session_status = 'refresh_failed',
            session_error = $1,
            session_checked_at = NOW()
        WHERE id = $2
        "#,
    )
    .bind(message)
    .bind(account_id)
    .execute(db)
    .await
    .map_err(|error| format!("Update Codex refresh failure state failed: {error}"))?;

    Ok(())
}

async fn monitor_device_login(
    db: sqlx::PgPool,
    config: AppConfig,
    sessions: CodexLoginSessionStore,
    session_id: String,
    account_id: i32,
    home_dir: PathBuf,
) {
    let mut child = match Command::new("codex")
        .arg("login")
        .env("HOME", &home_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            update_login_session(
                &sessions,
                &session_id,
                "failed",
                None,
                None,
                Some(format!("Failed to start Codex CLI login: {error}")),
            )
            .await;
            return;
        }
    };

    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    if let Some(stdout) = child.stdout.take() {
        tokio::spawn(read_process_lines(stdout, tx.clone()));
    }
    if let Some(stderr) = child.stderr.take() {
        tokio::spawn(read_process_lines(stderr, tx.clone()));
    }
    drop(tx);

    let mut combined_output = String::new();
    let mut output_stream_open = true;

    loop {
        tokio::select! {
            maybe_line = rx.recv(), if output_stream_open => {
                let Some(line) = maybe_line else {
                    output_stream_open = false;
                    continue;
                };

                let sanitized = strip_ansi_escape_sequences(&line);
                if sanitized.trim().is_empty() {
                    continue;
                }

                if combined_output.len() < 8192 {
                    combined_output.push_str(&sanitized);
                    combined_output.push('\n');
                }

                if let Some(url) = extract_auth_url(&sanitized) {
                    update_login_session(
                        &sessions,
                        &session_id,
                        "awaiting_user",
                        Some(url),
                        None,
                        Some("Open the login URL in your browser and finish the Codex sign-in flow. Keep this dialog open until the account is connected.".to_string()),
                    ).await;
                } else if let Some(code) = extract_device_user_code(&sanitized) {
                    update_login_session(
                        &sessions,
                        &session_id,
                        "awaiting_user",
                        None,
                        Some(code),
                        Some("Open the verification URL, enter the code, then keep this dialog open until the account is connected.".to_string()),
                    ).await;
                }
            }
            wait_result = child.wait() => {
                match wait_result {
                    Ok(status) if status.success() => {
                        match load_native_auth_tokens(&home_dir.join(".codex").join("auth.json")) {
                            Ok(tokens) => {
                                let persist_result = async {
                                    persist_tokens(&db, account_id, &tokens).await?;
                                    accounts::update_account(
                                        &db,
                                        account_id,
                                        None,
                                        Some(true),
                                        None,
                                        Some(Some(home_dir.to_string_lossy().to_string())),
                                    )
                                    .await
                                    .map_err(|error| format!("Persist Codex home path failed: {error}"))?;
                                    Ok::<(), String>(())
                                }
                                .await;

                                match persist_result {
                                    Ok(()) => {
                                        update_login_session(
                                            &sessions,
                                            &session_id,
                                            "completed",
                                            None,
                                            None,
                                            Some("Codex account connected successfully.".to_string()),
                                        ).await;
                                    }
                                    Err(error) => {
                                        update_login_session(&sessions, &session_id, "failed", None, None, Some(error)).await;
                                    }
                                }
                            }
                            Err(error) => {
                                update_login_session(&sessions, &session_id, "failed", None, None, Some(error)).await;
                            }
                        }
                    }
                    Ok(_) => {
                        update_login_session(
                            &sessions,
                            &session_id,
                            "failed",
                            None,
                            None,
                            Some(failure_message_from_output(&combined_output)),
                        ).await;
                    }
                    Err(error) => {
                        update_login_session(
                            &sessions,
                            &session_id,
                            "failed",
                            None,
                            None,
                            Some(format!("Waiting for Codex login process failed: {error}")),
                        ).await;
                    }
                }
                break;
            }
        }
    }

    let _ = config;
}

async fn read_process_lines<R>(reader: R, tx: mpsc::UnboundedSender<String>)
where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    let mut lines = BufReader::new(reader).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        if tx.send(line).is_err() {
            return;
        }
    }
}

fn load_native_auth_tokens(path: &Path) -> Result<CodexTokens, String> {
    let raw = std::fs::read_to_string(path)
        .map_err(|error| format!("Read Codex auth file failed: {error}"))?;
    let auth: NativeCodexAuthFile =
        serde_json::from_str(&raw).map_err(|error| format!("Invalid Codex auth file: {error}"))?;
    let access_token = auth.tokens.access_token.clone();

    let mut tokens = CodexTokens {
        access_token,
        refresh_token: auth.tokens.refresh_token,
        id_token: auth.tokens.id_token,
        account_id: auth.tokens.account_id,
        email: None,
        expires_at: jwt_expiry_rfc3339(&auth.tokens.access_token),
        last_refresh_at: auth.last_refresh.or_else(|| Some(Utc::now().to_rfc3339())),
        scope: jwt_scope(&auth.tokens.access_token),
        token_type: Some("Bearer".to_string()),
    };
    enrich_identity_from_id_token(&mut tokens);
    enrich_identity_from_access_token(&mut tokens);

    if tokens.access_token.trim().is_empty() {
        return Err("Codex auth file is missing access_token".to_string());
    }

    Ok(tokens)
}

async fn cleanup_stale_login_sessions(sessions: &CodexLoginSessionStore) {
    let mut guard = sessions.write().await;
    guard.retain(|_, session| !is_login_session_stale(session));
}

async fn current_login_session_for_account(
    sessions: &CodexLoginSessionStore,
    account_id: i32,
) -> Option<CodexLoginSession> {
    sessions
        .read()
        .await
        .values()
        .filter(|session| session.account_id == account_id)
        .max_by_key(|session| session.started_at)
        .cloned()
}

async fn get_login_session(
    sessions: &CodexLoginSessionStore,
    session_id: &str,
) -> Option<CodexLoginSession> {
    sessions.read().await.get(session_id).cloned()
}

async fn update_login_session(
    sessions: &CodexLoginSessionStore,
    session_id: &str,
    status: &str,
    verification_url: Option<String>,
    user_code: Option<String>,
    message: Option<String>,
) {
    let mut guard = sessions.write().await;
    if let Some(session) = guard.get_mut(session_id) {
        session.status = status.to_string();
        if let Some(url) = verification_url {
            session.verification_url = url;
            session.expires_at = Some(Utc::now() + Duration::minutes(CODEX_LOGIN_EXPIRY_MINUTES));
        }
        if let Some(code) = user_code {
            session.user_code = Some(code);
            session.expires_at = Some(Utc::now() + Duration::minutes(CODEX_LOGIN_EXPIRY_MINUTES));
        }
        if let Some(message) = message {
            session.message = Some(message);
        }
    }
}

fn failure_message_from_output(output: &str) -> String {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return "Codex login did not complete successfully.".to_string();
    }
    trimmed
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("Codex login did not complete successfully.")
        .trim()
        .chars()
        .take(300)
        .collect()
}

fn codex_account_home_dir(config: &AppConfig, account_id: i32) -> PathBuf {
    let data_root = Path::new(&config.data_dir)
        .parent()
        .unwrap_or_else(|| Path::new("data"));
    data_root
        .join("codex-accounts")
        .join(account_id.to_string())
        .join("home")
}

fn oauth_client_id(config: &AppConfig) -> String {
    config
        .codex_oauth_client_id
        .clone()
        .unwrap_or_else(|| DEFAULT_CODEX_OAUTH_CLIENT_ID.to_string())
}

async fn parse_token_response(response: reqwest::Response) -> Result<CodexTokens, String> {
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| format!("Read Codex token response failed: {error}"))?;

    if !status.is_success() {
        let preview = body.chars().take(300).collect::<String>();
        return Err(format!(
            "Codex OAuth endpoint returned {}: {}",
            status, preview
        ));
    }

    let token_response: CodexTokenResponse = serde_json::from_str(&body)
        .map_err(|error| format!("Invalid Codex token response: {error}"))?;
    let mut tokens = token_response_to_tokens(token_response);
    enrich_identity_from_id_token(&mut tokens);
    enrich_identity_from_access_token(&mut tokens);
    Ok(tokens)
}

fn token_response_to_tokens(response: CodexTokenResponse) -> CodexTokens {
    CodexTokens {
        access_token: response.access_token,
        refresh_token: response.refresh_token,
        id_token: response.id_token,
        account_id: None,
        email: None,
        expires_at: response
            .expires_in
            .map(|seconds| (Utc::now() + Duration::seconds(seconds)).to_rfc3339()),
        last_refresh_at: Some(Utc::now().to_rfc3339()),
        scope: response.scope,
        token_type: response.token_type,
    }
}

fn merge_refreshed_tokens(previous: &CodexTokens, refreshed: CodexTokens) -> CodexTokens {
    CodexTokens {
        access_token: refreshed.access_token,
        refresh_token: refreshed
            .refresh_token
            .or_else(|| previous.refresh_token.clone()),
        id_token: refreshed.id_token.or_else(|| previous.id_token.clone()),
        account_id: refreshed.account_id.or_else(|| previous.account_id.clone()),
        email: refreshed.email.or_else(|| previous.email.clone()),
        expires_at: refreshed.expires_at.or_else(|| previous.expires_at.clone()),
        last_refresh_at: refreshed
            .last_refresh_at
            .or_else(|| previous.last_refresh_at.clone()),
        scope: refreshed.scope.or_else(|| previous.scope.clone()),
        token_type: refreshed.token_type.or_else(|| previous.token_type.clone()),
    }
}

fn enrich_identity_from_id_token(tokens: &mut CodexTokens) {
    let Some(id_token) = tokens.id_token.as_deref() else {
        return;
    };
    let Some(value) = decode_jwt_payload(id_token) else {
        return;
    };

    if tokens.email.is_none() {
        tokens.email = value
            .get("email")
            .and_then(Value::as_str)
            .map(str::to_string);
    }
    if tokens.account_id.is_none() {
        tokens.account_id = value.get("sub").and_then(Value::as_str).map(str::to_string);
    }
}

fn enrich_identity_from_access_token(tokens: &mut CodexTokens) {
    let Some(value) = decode_jwt_payload(&tokens.access_token) else {
        return;
    };

    if tokens.expires_at.is_none() {
        tokens.expires_at = value
            .get("exp")
            .and_then(Value::as_i64)
            .and_then(|seconds| Utc.timestamp_opt(seconds, 0).single())
            .map(|value| value.to_rfc3339());
    }

    if tokens.scope.is_none() {
        tokens.scope = value
            .get("scp")
            .and_then(Value::as_array)
            .map(|scopes| {
                scopes
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .filter(|scope| !scope.is_empty());
    }

    if tokens.email.is_none() {
        tokens.email = value
            .get("https://api.openai.com/profile")
            .and_then(|profile| profile.get("email"))
            .and_then(Value::as_str)
            .map(str::to_string);
    }

    if tokens.account_id.is_none() {
        tokens.account_id = value
            .get("https://api.openai.com/auth")
            .and_then(|auth| auth.get("chatgpt_account_id"))
            .and_then(Value::as_str)
            .map(str::to_string);
    }
}

fn jwt_expiry_rfc3339(token: &str) -> Option<String> {
    decode_jwt_payload(token)?
        .get("exp")
        .and_then(Value::as_i64)
        .and_then(|seconds| Utc.timestamp_opt(seconds, 0).single())
        .map(|value| value.to_rfc3339())
}

fn jwt_scope(token: &str) -> Option<String> {
    decode_jwt_payload(token)?
        .get("scp")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(" ")
        })
        .filter(|scope| !scope.is_empty())
}

fn decode_jwt_payload(token: &str) -> Option<Value> {
    let payload_segment = token.split('.').nth(1)?;
    let decoded = URL_SAFE_NO_PAD.decode(payload_segment.as_bytes()).ok()?;
    serde_json::from_slice::<Value>(&decoded).ok()
}

fn is_login_session_stale(session: &CodexLoginSession) -> bool {
    session.started_at < Utc::now() - Duration::minutes(CODEX_LOGIN_STALE_MINUTES)
        || matches!(
            session.expires_at,
            Some(expires_at) if expires_at < Utc::now() - Duration::minutes(2)
        )
}

fn is_terminal_login_status(status: &str) -> bool {
    matches!(status, "completed" | "failed" | "expired")
}

fn extract_device_user_code(line: &str) -> Option<String> {
    line.split_whitespace()
        .map(str::trim)
        .find(|candidate| is_device_user_code(candidate))
        .map(str::to_string)
}

fn extract_auth_url(line: &str) -> Option<String> {
    line.split_whitespace()
        .find(|value| value.starts_with("https://auth.openai.com/"))
        .map(|value| {
            value
                .trim_matches(|ch: char| matches!(ch, '"' | '\'' | '(' | ')' | '[' | ']'))
                .to_string()
        })
}

fn validate_manual_callback_url(
    callback_url: &str,
    session: &CodexLoginSession,
) -> Result<Url, String> {
    let parsed = Url::parse(callback_url.trim())
        .map_err(|error| format!("Invalid callback URL: {error}"))?;

    if parsed.scheme() != "http" {
        return Err("Callback URL must use http://".to_string());
    }

    let host = parsed
        .host_str()
        .ok_or_else(|| "Callback URL is missing host".to_string())?;
    if host != "localhost" && host != "127.0.0.1" {
        return Err("Callback URL host must be localhost or 127.0.0.1".to_string());
    }
    if parsed.port_or_known_default() != Some(1455) {
        return Err("Callback URL port must be 1455".to_string());
    }
    if parsed.path() != "/auth/callback" {
        return Err("Callback URL path must be /auth/callback".to_string());
    }

    let code = parsed
        .query_pairs()
        .find(|(key, _)| key == "code")
        .map(|(_, value)| value.into_owned())
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "Callback URL is missing code".to_string())?;
    if !code.starts_with("ac_") {
        return Err("Callback code does not look like a Codex auth code".to_string());
    }

    let callback_state = parsed
        .query_pairs()
        .find(|(key, _)| key == "state")
        .map(|(_, value)| value.into_owned())
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "Callback URL is missing state".to_string())?;

    let session_state = Url::parse(&session.verification_url)
        .ok()
        .and_then(|url| {
            url.query_pairs()
                .find(|(key, _)| key == "state")
                .map(|(_, value)| value.into_owned())
        })
        .ok_or_else(|| "Current Codex login session is missing state".to_string())?;

    if callback_state != session_state {
        return Err(
            "Callback URL state does not match the current Codex login session".to_string(),
        );
    }

    let mut local_callback = parsed;
    local_callback
        .set_host(Some("127.0.0.1"))
        .map_err(|error| format!("Failed to normalize callback host: {error}"))?;
    Ok(local_callback)
}

fn is_device_user_code(value: &str) -> bool {
    let mut parts = value.trim().split('-');
    let Some(left) = parts.next() else {
        return false;
    };
    let Some(right) = parts.next() else {
        return false;
    };
    if parts.next().is_some() {
        return false;
    }
    left.len() == 4
        && right.len() == 5
        && left
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit())
        && right
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit())
}

fn strip_ansi_escape_sequences(line: &str) -> String {
    let mut output = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '\u{1b}' {
            output.push(ch);
            continue;
        }

        if chars.peek() == Some(&'[') {
            let _ = chars.next();
            for next in chars.by_ref() {
                if ('@'..='~').contains(&next) {
                    break;
                }
            }
        }
    }

    output
}

fn random_token(length: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

impl CodexLoginSession {
    fn to_view(&self) -> CodexLoginSessionView {
        CodexLoginSessionView {
            session_id: self.session_id.clone(),
            account_id: self.account_id,
            status: self.status.clone(),
            verification_url: self.verification_url.clone(),
            user_code: self.user_code.clone(),
            expires_at: self.expires_at.map(|value| value.to_rfc3339()),
            command: self.command.clone(),
            message: self.message.clone(),
        }
    }
}
