use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::grok::client::GrokRequestError;
use crate::grok::media_response_parser::GeneratedImageAsset;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub enum ChatStreamEvent {
    Token(String),
    Thinking(String),
    Error(String),
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedAsset {
    pub id: String,
    pub url: String,
}

#[derive(Debug, Clone)]
pub enum ProviderError {
    RateLimited,
    Unauthorized,
    CfBlocked,
    ProxyFailed(String),
    Network(String),
}

impl Display for ProviderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RateLimited => write!(f, "Rate limited"),
            Self::Unauthorized => write!(f, "Unauthorized"),
            Self::CfBlocked => write!(f, "Cloudflare blocked"),
            Self::ProxyFailed(message) => write!(f, "{message}"),
            Self::Network(message) => write!(f, "{message}"),
        }
    }
}

impl From<GrokRequestError> for ProviderError {
    fn from(value: GrokRequestError) -> Self {
        match value {
            GrokRequestError::RateLimited => Self::RateLimited,
            GrokRequestError::Unauthorized => Self::Unauthorized,
            GrokRequestError::CfBlocked => Self::CfBlocked,
            GrokRequestError::ProxyFailed(message) => Self::ProxyFailed(message),
            other => Self::Network(other.to_string()),
        }
    }
}

impl From<GeneratedImageAsset> for GeneratedAsset {
    fn from(value: GeneratedImageAsset) -> Self {
        Self {
            id: value.id,
            url: value.url,
        }
    }
}


impl From<ProviderError> for AppError {
    fn from(value: ProviderError) -> Self {
        match value {
            ProviderError::RateLimited => AppError::GrokApi("Rate limited".into()),
            ProviderError::Unauthorized => AppError::GrokApi("Unauthorized".into()),
            ProviderError::CfBlocked => AppError::GrokApi("Cloudflare blocked".into()),
            ProviderError::ProxyFailed(message) => AppError::GrokApi(message),
            ProviderError::Network(message) => AppError::GrokApi(message),
        }
    }
}
