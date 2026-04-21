use async_trait::async_trait;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::account::pool::CurrentAccount;
use crate::config::AppConfig;
use crate::providers::types::{ChatMessage, ChatStreamEvent, ProviderCapabilities, ProviderError};

#[async_trait]
pub trait ChatProvider: Send + Sync {
    fn capabilities(&self) -> ProviderCapabilities;

    async fn prepare_account_for_request(
        &self,
        _db: &sqlx::PgPool,
        _config: &AppConfig,
        account: CurrentAccount,
    ) -> Option<CurrentAccount> {
        Some(account)
    }

    async fn chat_stream(
        &self,
        account: &CurrentAccount,
        model: &str,
        messages: &[ChatMessage],
        system_prompt: &str,
    ) -> Result<UnboundedReceiver<ChatStreamEvent>, ProviderError>;
}
