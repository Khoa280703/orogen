use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use tokio::sync::{Mutex, RwLock};

use crate::account::types::{
    AUTH_MODE_GROK_COOKIES, AccountCredential, AccountEntry, PROVIDER_CODEX, PROVIDER_GROK,
    ROUTING_STATE_AUTH_INVALID, ROUTING_STATE_CANDIDATE, ROUTING_STATE_COOLING_DOWN,
    ROUTING_STATE_HEALTHY, ROUTING_STATE_PAUSED, ROUTING_STATE_REFRESH_FAILED,
};
use crate::config::AppConfig;
use crate::db::{accounts, proxies};
use crate::providers::ProviderRegistry;

#[derive(Clone)]
pub struct AccountPool {
    db: sqlx::PgPool,
    config: AppConfig,
    providers: ProviderRegistry,
    current_indices: Arc<RwLock<HashMap<String, usize>>>,
    last_selected_account_ids: Arc<RwLock<HashMap<String, i32>>>,
    provider_selection_locks: Arc<Mutex<HashMap<String, Arc<Mutex<()>>>>>,
}

#[derive(Clone)]
pub struct CurrentAccount {
    pub id: Option<i32>,
    pub name: String,
    pub provider_slug: String,
    pub account_label: Option<String>,
    pub external_account_id: Option<String>,
    pub auth_mode: Option<String>,
    pub credential: AccountCredential,
    pub proxy_id: Option<i32>,
    pub proxy_url: Option<String>,
}

impl CurrentAccount {
    pub fn grok_cookies(&self) -> Result<&crate::account::types::GrokCookies, String> {
        self.credential
            .as_grok_cookies()
            .ok_or_else(|| format!("Account {} does not have Grok cookies", self.name))
    }

    pub fn codex_tokens(&self) -> Result<&crate::account::types::CodexTokens, String> {
        self.credential
            .as_codex_tokens()
            .ok_or_else(|| format!("Account {} does not have Codex OAuth tokens", self.name))
    }
}

impl AccountPool {
    pub fn new(db: sqlx::PgPool, config: AppConfig, providers: ProviderRegistry) -> Self {
        Self {
            db,
            config,
            providers,
            current_indices: Arc::new(RwLock::new(HashMap::new())),
            last_selected_account_ids: Arc::new(RwLock::new(HashMap::new())),
            provider_selection_locks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_current(&self) -> Option<CurrentAccount> {
        self.get_current_for_provider(PROVIDER_GROK).await
    }

    pub async fn get_current_for_provider(&self, provider_slug: &str) -> Option<CurrentAccount> {
        let selection_lock = self.selection_lock_for_provider(provider_slug).await;
        let _selection_guard = selection_lock.lock().await;

        let accounts = self.runtime_accounts(provider_slug).await;
        if accounts.is_empty() {
            return None;
        }

        let start_index = {
            let mut indices = self.current_indices.write().await;
            let entry = indices.entry(provider_slug.to_string()).or_insert(0);
            *entry %= accounts.len();
            *entry
        };

        for offset in 0..accounts.len() {
            let index = (start_index + offset) % accounts.len();
            let candidate = accounts[index].clone();
            if let Some(prepared) = self.prepare_account_for_request(candidate).await {
                self.record_selected_account(provider_slug, accounts.len(), index, prepared.id)
                    .await;
                return Some(prepared);
            }
        }

        None
    }

    pub async fn rotate(&self) -> bool {
        self.rotate_provider(PROVIDER_GROK).await
    }

    pub async fn rotate_provider(&self, provider_slug: &str) -> bool {
        let selection_lock = self.selection_lock_for_provider(provider_slug).await;
        let _selection_guard = selection_lock.lock().await;

        let count = self.active_account_count(provider_slug).await;
        if count <= 1 {
            return false;
        }

        let mut indices = self.current_indices.write().await;
        let entry = indices.entry(provider_slug.to_string()).or_insert(0);
        *entry = (*entry + 1) % count;
        true
    }

    pub async fn get_next_for_provider(
        &self,
        provider_slug: &str,
        current_account_id: Option<i32>,
        seen_account_ids: &HashSet<i32>,
    ) -> Option<CurrentAccount> {
        let selection_lock = self.selection_lock_for_provider(provider_slug).await;
        let _selection_guard = selection_lock.lock().await;

        let accounts = self.runtime_accounts(provider_slug).await;
        if accounts.is_empty() {
            return None;
        }

        let fallback_index = {
            let indices = self.current_indices.read().await;
            indices.get(provider_slug).copied().unwrap_or(0)
        };
        let start_index = next_runtime_account_start_index(&accounts, current_account_id)
            .unwrap_or(fallback_index);

        for offset in 0..accounts.len() {
            let index = (start_index + offset) % accounts.len();
            let candidate = accounts[index].clone();
            if candidate
                .id
                .is_some_and(|candidate_id| seen_account_ids.contains(&candidate_id))
            {
                continue;
            }

            if let Some(prepared) = self.prepare_account_for_request(candidate).await {
                self.record_selected_account(provider_slug, accounts.len(), index, prepared.id)
                    .await;
                return Some(prepared);
            }
        }

        None
    }

    pub async fn mark_used(&self) {
        self.mark_used_for_provider(PROVIDER_GROK).await;
    }

    pub async fn mark_used_for_provider(&self, provider_slug: &str) {
        if let Some(id) = self.last_selected_account_id(provider_slug).await {
            let _ = accounts::increment_request_count(&self.db, id).await;
        }
    }

    pub async fn mark_expired(&self) -> Option<String> {
        let account = self.get_current().await?;
        if let Some(id) = account.id {
            let _ = accounts::update_account(&self.db, id, None, Some(false), None, None).await;
            let mut indices = self.current_indices.write().await;
            indices.insert(account.provider_slug.clone(), 0);
            Some(account.name)
        } else {
            None
        }
    }

    pub async fn get_all_accounts(&self) -> Vec<AccountEntry> {
        match accounts::list_accounts(&self.db).await {
            Ok(rows) => rows
                .into_iter()
                .filter_map(|row| {
                    let credential = credential_from_row(
                        &row.provider_slug,
                        row.credential_payload.as_ref(),
                        &row.cookies,
                    )
                    .ok()?;

                    Some(AccountEntry {
                        name: row.name,
                        provider_slug: row.provider_slug,
                        credential_preview: credential.to_preview(),
                        account_label: row.account_label,
                        external_account_id: row.external_account_id,
                        active: row.active.unwrap_or(true),
                        request_count: row.request_count.unwrap_or(0) as u64,
                        last_used: row.last_used.map(|value| value.to_rfc3339()),
                        proxy_url: None,
                        fail_count: row.fail_count.unwrap_or(0) as u32,
                        success_count: row.success_count.unwrap_or(0) as u64,
                    })
                })
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    pub async fn mark_success(&self) {
        self.mark_success_for_provider(PROVIDER_GROK).await;
    }

    pub async fn mark_success_for_provider(&self, provider_slug: &str) {
        if let Some(id) = self.last_selected_account_id(provider_slug).await {
            let _ = accounts::update_health_counts(&self.db, id, true).await;
        }
    }

    pub async fn mark_failure(&self) -> bool {
        self.mark_failure_for_provider(PROVIDER_GROK).await
    }

    pub async fn mark_failure_for_provider(&self, provider_slug: &str) -> bool {
        if let Some(id) = self.last_selected_account_id(provider_slug).await {
            return accounts::update_health_counts(&self.db, id, false)
                .await
                .unwrap_or(false);
        }
        false
    }

    pub async fn mark_rate_limited(&self) {
        self.mark_rate_limited_for_provider(PROVIDER_GROK).await;
    }

    pub async fn mark_rate_limited_for_provider(&self, provider_slug: &str) {
        if let Some(id) = self.last_selected_account_id(provider_slug).await {
            let _ = accounts::record_rate_limited_attempt(&self.db, id).await;
        }
    }

    pub async fn update_account_health(&self, name: &str, success: bool) {
        if let Ok(rows) = accounts::list_accounts(&self.db).await {
            if let Some(account) = rows.into_iter().find(|row| row.name == name) {
                let _ = accounts::update_health_counts(&self.db, account.id, success).await;
            }
        }
    }

    async fn active_account_count(&self, provider_slug: &str) -> usize {
        self.runtime_accounts(provider_slug).await.len()
    }

    async fn selection_lock_for_provider(&self, provider_slug: &str) -> Arc<Mutex<()>> {
        let mut locks = self.provider_selection_locks.lock().await;
        locks
            .entry(provider_slug.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }

    async fn last_selected_account_id(&self, provider_slug: &str) -> Option<i32> {
        let selected = self.last_selected_account_ids.read().await;
        selected.get(provider_slug).copied()
    }

    async fn record_selected_account(
        &self,
        provider_slug: &str,
        account_count: usize,
        selected_index: usize,
        account_id: Option<i32>,
    ) {
        {
            let mut indices = self.current_indices.write().await;
            indices.insert(
                provider_slug.to_string(),
                (selected_index + 1) % account_count.max(1),
            );
        }

        let mut selected = self.last_selected_account_ids.write().await;
        match account_id {
            Some(id) => {
                selected.insert(provider_slug.to_string(), id);
            }
            None => {
                selected.remove(provider_slug);
            }
        }
    }

    async fn prepare_account_for_request(&self, account: CurrentAccount) -> Option<CurrentAccount> {
        if let Some(provider) = self.providers.chat_provider(&account.provider_slug) {
            return provider
                .prepare_account_for_request(&self.db, &self.config, account)
                .await;
        }

        Some(account)
    }

    async fn runtime_accounts(&self, provider_slug: &str) -> Vec<CurrentAccount> {
        let active_proxies = match proxies::list_active_proxies(&self.db).await {
            Ok(rows) => rows,
            Err(error) => {
                tracing::warn!(error = %error, "Failed to load active proxies from database");
                Vec::new()
            }
        };

        match accounts::list_runtime_accounts_by_provider(&self.db, provider_slug).await {
            Ok(rows) => {
                let mut runtime_accounts = Vec::new();
                let now = chrono::Utc::now();

                for (index, row) in rows.into_iter().enumerate() {
                    if !runtime_account_is_selectable(&row, now) {
                        continue;
                    }

                    let credential = match credential_from_row(
                        &row.provider_slug,
                        row.credential_payload.as_ref(),
                        &row.cookies,
                    ) {
                        Ok(credential) => credential,
                        Err(error) => {
                            tracing::warn!(
                                account_id = row.id,
                                account = row.name,
                                provider = row.provider_slug,
                                error = %error,
                                "Skipping account with invalid credential payload"
                            );
                            continue;
                        }
                    };

                    let auto_proxy = if row.proxy_id.is_none()
                        && supports_automatic_proxy_assignment(&row.provider_slug)
                        && !active_proxies.is_empty()
                    {
                        Some(&active_proxies[index % active_proxies.len()])
                    } else {
                        None
                    };

                    let derived_proxy_id =
                        row.proxy_id.or_else(|| auto_proxy.map(|proxy| proxy.id));
                    let derived_proxy_url = row
                        .proxy_url
                        .or_else(|| auto_proxy.map(|proxy| proxy.url.clone()));

                    if row.proxy_id.is_none() {
                        if let Some(proxy) = auto_proxy {
                            if let Err(error) =
                                accounts::assign_proxy_to_account(&self.db, row.id, Some(proxy.id))
                                    .await
                            {
                                tracing::warn!(
                                    account_id = row.id,
                                    proxy_id = proxy.id,
                                    error = %error,
                                    "Failed to persist auto-assigned proxy for runtime account"
                                );
                            }
                        }
                    }

                    runtime_accounts.push(CurrentAccount {
                        id: Some(row.id),
                        name: row.name,
                        provider_slug: row.provider_slug,
                        account_label: row.account_label,
                        external_account_id: row.external_account_id,
                        auth_mode: row
                            .auth_mode
                            .or_else(|| Some(AUTH_MODE_GROK_COOKIES.to_string())),
                        credential,
                        proxy_id: derived_proxy_id,
                        proxy_url: derived_proxy_url,
                    });
                }

                runtime_accounts
            }
            Err(error) => {
                tracing::warn!(
                    provider = provider_slug,
                    error = %error,
                    "Failed to load runtime accounts from database"
                );
                Vec::new()
            }
        }
    }
}

fn runtime_account_is_selectable(
    row: &crate::db::accounts::RuntimeAccountRow,
    now: chrono::DateTime<chrono::Utc>,
) -> bool {
    if matches!(
        row.routing_state.as_str(),
        ROUTING_STATE_PAUSED | ROUTING_STATE_AUTH_INVALID | ROUTING_STATE_REFRESH_FAILED
    ) {
        return false;
    }

    if matches!(
        row.session_status.as_deref(),
        Some("expired" | "needs_login" | "refresh_failed")
    ) {
        return false;
    }

    if row.routing_state == ROUTING_STATE_COOLING_DOWN {
        return row.cooldown_until.is_some_and(|until| until <= now);
    }

    matches!(
        row.routing_state.as_str(),
        ROUTING_STATE_HEALTHY | ROUTING_STATE_CANDIDATE
    )
}

fn next_runtime_account_start_index(
    accounts: &[CurrentAccount],
    current_account_id: Option<i32>,
) -> Option<usize> {
    let current_id = current_account_id?;
    let current_index = accounts
        .iter()
        .position(|account| account.id == Some(current_id))?;
    Some((current_index + 1) % accounts.len())
}

fn credential_from_row(
    provider_slug: &str,
    credential_payload: Option<&serde_json::Value>,
    legacy_cookies: &serde_json::Value,
) -> Result<AccountCredential, String> {
    if let Some(payload) = credential_payload {
        return AccountCredential::from_provider_value(provider_slug, payload);
    }

    if provider_slug == PROVIDER_GROK {
        return AccountCredential::from_provider_value(PROVIDER_GROK, legacy_cookies);
    }

    Err(format!(
        "Missing credential payload for provider {provider_slug}"
    ))
}

fn supports_automatic_proxy_assignment(provider_slug: &str) -> bool {
    matches!(provider_slug, PROVIDER_GROK | PROVIDER_CODEX)
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use serde_json::json;

    use crate::account::pool::{CurrentAccount, next_runtime_account_start_index};
    use crate::account::types::{AccountCredential, GrokCookies};
    use crate::db::accounts::RuntimeAccountRow;

    #[test]
    fn skips_accounts_in_cooldown_until_their_window_expires() {
        let now = Utc::now();
        let cooling = sample_runtime_row("cooling_down", Some(now + Duration::seconds(30)));
        let cooled = sample_runtime_row("cooling_down", Some(now - Duration::seconds(30)));

        assert!(!super::runtime_account_is_selectable(&cooling, now));
        assert!(super::runtime_account_is_selectable(&cooled, now));
    }

    #[test]
    fn skips_auth_invalid_and_refresh_failed_accounts() {
        let now = Utc::now();
        assert!(!super::runtime_account_is_selectable(
            &sample_runtime_row("auth_invalid", None),
            now,
        ));
        assert!(!super::runtime_account_is_selectable(
            &sample_runtime_row("refresh_failed", None),
            now,
        ));
        assert!(super::runtime_account_is_selectable(
            &sample_runtime_row("candidate", None),
            now,
        ));
    }

    #[test]
    fn next_start_index_advances_from_current_account() {
        let accounts = vec![
            sample_current_account(10, "a"),
            sample_current_account(11, "b"),
            sample_current_account(12, "c"),
        ];

        assert_eq!(
            next_runtime_account_start_index(&accounts, Some(10)),
            Some(1)
        );
        assert_eq!(
            next_runtime_account_start_index(&accounts, Some(12)),
            Some(0)
        );
    }

    #[test]
    fn next_start_index_returns_none_when_current_missing() {
        let accounts = vec![sample_current_account(10, "a")];

        assert_eq!(next_runtime_account_start_index(&accounts, Some(99)), None);
        assert_eq!(next_runtime_account_start_index(&accounts, None), None);
    }

    fn sample_runtime_row(
        routing_state: &str,
        cooldown_until: Option<chrono::DateTime<chrono::Utc>>,
    ) -> RuntimeAccountRow {
        RuntimeAccountRow {
            id: 1,
            name: "sample".into(),
            provider_slug: "codex".into(),
            account_label: None,
            external_account_id: None,
            auth_mode: Some("codex_oauth".into()),
            metadata: json!({}),
            cookies: json!({}),
            credential_type: Some("codex_oauth_tokens".into()),
            credential_payload: Some(json!({"access_token":"token"})),
            proxy_id: None,
            proxy_url: None,
            session_status: Some("healthy".into()),
            last_used: None,
            created_at: Some(Utc::now()),
            routing_state: routing_state.into(),
            cooldown_until,
            last_routing_error: None,
            rate_limit_streak: 0,
            auth_failure_streak: 0,
            refresh_failure_streak: 0,
        }
    }

    fn sample_current_account(id: i32, name: &str) -> CurrentAccount {
        CurrentAccount {
            id: Some(id),
            name: name.into(),
            provider_slug: "grok".into(),
            account_label: None,
            external_account_id: None,
            auth_mode: None,
            credential: AccountCredential::GrokCookies(GrokCookies {
                sso: "cookie".into(),
                sso_rw: None,
                cf_clearance: None,
                raw: Some("sso=cookie".into()),
                extra: std::collections::HashMap::new(),
            }),
            proxy_id: None,
            proxy_url: None,
        }
    }
}
