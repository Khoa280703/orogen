pub mod chat_provider;
pub mod codex_chat;
pub mod grok_chat;
pub mod grok_image;
pub mod image_provider;
pub mod types;

use std::collections::HashMap;
use std::sync::Arc;

use crate::grok::client::GrokClient;
use crate::providers::chat_provider::ChatProvider;
use crate::providers::codex_chat::CodexChatProvider;
use crate::providers::grok_chat::GrokChatProvider;
use crate::providers::grok_image::GrokImageProvider;
use crate::providers::image_provider::ImageProvider;
pub use crate::providers::types::{ChatMessage, ChatStreamEvent, GeneratedAsset, ProviderError};
use crate::services::codex_client::CodexClient;

#[derive(Clone, Default)]
pub struct ProviderRegistry {
    chat: Arc<HashMap<String, Arc<dyn ChatProvider>>>,
    images: Arc<HashMap<String, Arc<dyn ImageProvider>>>,
}

impl ProviderRegistry {
    pub fn new(grok: GrokClient, codex: CodexClient) -> Self {
        let mut chat: HashMap<String, Arc<dyn ChatProvider>> = HashMap::new();
        let mut images: HashMap<String, Arc<dyn ImageProvider>> = HashMap::new();

        chat.insert("grok".to_string(), Arc::new(GrokChatProvider::new(grok)));
        chat.insert("codex".to_string(), Arc::new(CodexChatProvider::new(codex)));
        images.insert("grok".to_string(), Arc::new(GrokImageProvider));

        Self {
            chat: Arc::new(chat),
            images: Arc::new(images),
        }
    }

    pub fn with_grok(grok: GrokClient) -> Self {
        Self::new(grok, CodexClient::default())
    }

    pub fn chat_provider(&self, slug: &str) -> Option<Arc<dyn ChatProvider>> {
        self.chat.get(slug).cloned()
    }

    pub fn image_provider(&self, slug: &str) -> Option<Arc<dyn ImageProvider>> {
        self.images.get(slug).cloned()
    }

    pub fn get_chat_provider(&self, slug: &str) -> Option<Arc<dyn ChatProvider>> {
        self.chat_provider(slug)
    }

    pub fn get_image_provider(&self, slug: &str) -> Option<Arc<dyn ImageProvider>> {
        self.image_provider(slug)
    }
}
