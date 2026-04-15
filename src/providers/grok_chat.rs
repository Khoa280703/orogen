use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::account::types::GrokCookies;
use crate::grok::client::{GrokClient, StreamEvent};
use crate::grok::output_sanitizer::OutputSanitizer;
use crate::grok::types::GrokRequest;
use crate::providers::chat_provider::ChatProvider;
use crate::providers::types::{ChatMessage, ChatStreamEvent, ProviderError};

#[derive(Clone)]
pub struct GrokChatProvider {
    client: GrokClient,
}

impl GrokChatProvider {
    pub fn new(client: GrokClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ChatProvider for GrokChatProvider {
    async fn chat_stream(
        &self,
        cookies: &GrokCookies,
        proxy_url: Option<&String>,
        model: &str,
        messages: &[ChatMessage],
        system_prompt: &str,
    ) -> Result<mpsc::UnboundedReceiver<ChatStreamEvent>, ProviderError> {
        let payload = GrokRequest::new(
            flatten_messages(messages),
            model.to_string(),
            is_reasoning_model(model),
            system_prompt.to_string(),
        );
        let mut upstream = self
            .client
            .send_request_stream(cookies, &payload, proxy_url)
            .await
            .map_err(ProviderError::from)?;

        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            let mut token_sanitizer = OutputSanitizer::new();
            let mut thinking_sanitizer = OutputSanitizer::new();

            while let Some(event) = upstream.recv().await {
                let mapped = match event {
                    StreamEvent::Event(crate::grok::types::GrokStreamEvent::Token(token)) => {
                        let sanitized = token_sanitizer.process(&token);
                        if sanitized.is_empty() {
                            continue;
                        }
                        ChatStreamEvent::Token(sanitized)
                    }
                    StreamEvent::Event(crate::grok::types::GrokStreamEvent::Thinking(thinking)) => {
                        let sanitized = thinking_sanitizer.process(&thinking);
                        if sanitized.is_empty() {
                            continue;
                        }
                        ChatStreamEvent::Thinking(sanitized)
                    }
                    StreamEvent::Error(error) => ChatStreamEvent::Error(error),
                    StreamEvent::Done => ChatStreamEvent::Done,
                    StreamEvent::Event(_) => continue,
                };

                if tx.send(mapped).is_err() {
                    return;
                }
            }
        });

        Ok(rx)
    }
}

fn is_reasoning_model(model: &str) -> bool {
    let lowered = model.to_ascii_lowercase();
    lowered.contains("thinking") || lowered.contains("reasoning")
}

fn flatten_messages(messages: &[ChatMessage]) -> String {
    let mut chat_parts = Vec::new();
    for message in messages {
        match message.role.as_str() {
            "assistant" => chat_parts.push(format!("[Assistant]\n{}", message.content)),
            "system" => chat_parts.push(format!("[System]\n{}", message.content)),
            _ => chat_parts.push(format!("[User]\n{}", message.content)),
        }
    }

    if chat_parts.len() == 1 && messages.last().is_some_and(|message| message.role == "user") {
        messages
            .last()
            .map(|message| message.content.clone())
            .unwrap_or_default()
    } else {
        chat_parts.join("\n\n")
    }
}
