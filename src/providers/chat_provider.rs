use async_trait::async_trait;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::account::types::GrokCookies;
use crate::providers::types::{ChatMessage, ChatStreamEvent, ProviderError};

#[async_trait]
pub trait ChatProvider: Send + Sync {
    async fn chat_stream(
        &self,
        cookies: &GrokCookies,
        proxy_url: Option<&String>,
        model: &str,
        messages: &[ChatMessage],
        system_prompt: &str,
    ) -> Result<UnboundedReceiver<ChatStreamEvent>, ProviderError>;
}
