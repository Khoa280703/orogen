use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::account::pool::CurrentAccount;
use crate::config::AppConfig;
use crate::providers::chat_provider::ChatProvider;
use crate::providers::types::{
    ChatMessage, ChatStreamEvent, ProviderAuthMode, ProviderCapabilities, ProviderError,
};
use crate::services::codex_client::CodexClient;
use crate::services::codex_oauth;

const CODEX_CHAT_CAPABILITIES: ProviderCapabilities = ProviderCapabilities {
    auth_mode: ProviderAuthMode::OAuthToken,
    supports_chat_streaming: true,
    supports_proxy: true,
    supports_responses_api: true,
};

#[derive(Clone, Default)]
pub struct CodexChatProvider {
    client: CodexClient,
}

impl CodexChatProvider {
    pub fn new(client: CodexClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ChatProvider for CodexChatProvider {
    fn capabilities(&self) -> ProviderCapabilities {
        CODEX_CHAT_CAPABILITIES
    }

    async fn prepare_account_for_request(
        &self,
        db: &sqlx::PgPool,
        config: &AppConfig,
        account: CurrentAccount,
    ) -> Option<CurrentAccount> {
        let tokens = match account.codex_tokens() {
            Ok(tokens) => tokens.clone(),
            Err(error) => {
                tracing::warn!(account = account.name, %error, "Invalid Codex account payload");
                return None;
            }
        };

        if !tokens.should_refresh(120) && !tokens.is_expired() {
            return Some(account);
        }

        let account_id = account.id?;
        match codex_oauth::refresh_account_tokens(db, config, account_id, &tokens).await {
            Ok(refreshed) => Some(CurrentAccount {
                credential: crate::account::types::AccountCredential::CodexTokens(refreshed),
                ..account
            }),
            Err(error) => {
                let _ = codex_oauth::mark_refresh_failed(db, account_id, &error).await;
                tracing::warn!(
                    account_id,
                    account = account.name,
                    %error,
                    "Failed to refresh Codex tokens before request"
                );
                None
            }
        }
    }

    async fn chat_stream(
        &self,
        account: &CurrentAccount,
        model: &str,
        messages: &[ChatMessage],
        system_prompt: &str,
    ) -> Result<mpsc::UnboundedReceiver<ChatStreamEvent>, ProviderError> {
        self.client
            .send_request_stream(account, model, messages, system_prompt)
            .await
    }
}
