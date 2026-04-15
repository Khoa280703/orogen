use std::sync::Arc;

use tokio::sync::RwLock;

use crate::account::types::{AccountEntry, GrokCookies};
use crate::db::{accounts, proxies};

#[derive(Clone)]
pub struct AccountPool {
    db: sqlx::PgPool,
    current_index: Arc<RwLock<usize>>,
}

#[derive(Clone)]
pub struct CurrentAccount {
    pub id: Option<i32>,
    pub name: String,
    pub cookies: GrokCookies,
    pub proxy_id: Option<i32>,
    pub proxy_url: Option<String>,
}

impl AccountPool {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self {
            db,
            current_index: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn get_current(&self) -> Option<CurrentAccount> {
        let accounts = self.runtime_accounts().await;
        if accounts.is_empty() {
            return None;
        }

        let mut idx = self.current_index.write().await;
        *idx %= accounts.len();
        Some(accounts[*idx].clone())
    }

    pub async fn rotate(&self) -> bool {
        let count = self.active_account_count().await;
        if count <= 1 {
            return false;
        }

        let mut idx = self.current_index.write().await;
        *idx = (*idx + 1) % count;
        true
    }

    pub async fn mark_used(&self) {
        if let Some(account) = self.get_current().await {
            if let Some(id) = account.id {
                let _ = accounts::increment_request_count(&self.db, id).await;
            }
        }
    }

    pub async fn mark_expired(&self) -> Option<String> {
        let account = self.get_current().await?;
        if let Some(id) = account.id {
            let _ = accounts::update_account(&self.db, id, None, Some(false), None, None).await;
            *self.current_index.write().await = 0;
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
                    let cookies: GrokCookies = serde_json::from_value(row.cookies).ok()?;
                    Some(AccountEntry {
                        name: row.name,
                        cookies,
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
        if let Some(account) = self.get_current().await {
            if let Some(id) = account.id {
                let _ = accounts::update_health_counts(&self.db, id, true).await;
            }
        }
    }

    pub async fn mark_failure(&self) -> bool {
        if let Some(account) = self.get_current().await {
            if let Some(id) = account.id {
                return accounts::update_health_counts(&self.db, id, false)
                    .await
                    .unwrap_or(false);
            }
        }
        false
    }

    pub async fn mark_rate_limited(&self) {
        if let Some(account) = self.get_current().await {
            if let Some(id) = account.id {
                let _ = accounts::record_rate_limited_attempt(&self.db, id).await;
            }
        }
    }

    pub async fn update_account_health(&self, name: &str, success: bool) {
        if let Ok(rows) = accounts::list_accounts(&self.db).await {
            if let Some(account) = rows.into_iter().find(|row| row.name == name) {
                let _ = accounts::update_health_counts(&self.db, account.id, success).await;
            }
        }
    }

    async fn active_account_count(&self) -> usize {
        self.runtime_accounts().await.len()
    }

    async fn runtime_accounts(&self) -> Vec<CurrentAccount> {
        let active_proxies = match proxies::list_active_proxies(&self.db).await {
            Ok(rows) => rows,
            Err(error) => {
                tracing::warn!(error = %error, "Failed to load active proxies from database");
                Vec::new()
            }
        };

        match accounts::list_runtime_accounts(&self.db).await {
            Ok(rows) => rows
                .into_iter()
                .enumerate()
                .filter_map(|(index, row)| {
                    let cookies = match serde_json::from_value::<GrokCookies>(row.cookies) {
                        Ok(cookies) => cookies,
                        Err(error) => {
                            tracing::warn!(
                                account_id = row.id,
                                account = row.name,
                                error = %error,
                                "Skipping account with invalid cookies payload"
                            );
                            return None;
                        }
                    };

                    Some(CurrentAccount {
                        id: Some(row.id),
                        name: row.name,
                        cookies,
                        proxy_id: row.proxy_id.or_else(|| {
                            if active_proxies.is_empty() {
                                None
                            } else {
                                Some(active_proxies[index % active_proxies.len()].id)
                            }
                        }),
                        proxy_url: row.proxy_url.or_else(|| {
                            if active_proxies.is_empty() {
                                None
                            } else {
                                Some(active_proxies[index % active_proxies.len()].url.clone())
                            }
                        }),
                    })
                })
                .collect(),
            Err(error) => {
                tracing::warn!(error = %error, "Failed to load runtime accounts from database");
                Vec::new()
            }
        }
    }
}
