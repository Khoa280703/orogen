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
    Error(ProviderError),
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedAsset {
    pub id: String,
    pub url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderAuthMode {
    CookieSession,
    OAuthToken,
}

#[derive(Debug, Clone, Copy)]
pub struct ProviderCapabilities {
    pub auth_mode: ProviderAuthMode,
    pub supports_chat_streaming: bool,
    pub supports_proxy: bool,
    pub supports_responses_api: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderRoutingDisposition {
    RetryNextAccount,
    ExpireAccount,
    DeactivateProxy,
    DoNotRetry,
}

#[derive(Debug, Clone)]
pub enum ProviderError {
    RateLimited,
    Unauthorized,
    CfBlocked,
    ProxyFailed(String),
    UpstreamTransient(String),
    Network(String),
}

impl Display for ProviderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RateLimited => write!(f, "Rate limited"),
            Self::Unauthorized => write!(f, "Unauthorized"),
            Self::CfBlocked => write!(f, "Cloudflare blocked"),
            Self::ProxyFailed(message) => write!(f, "{message}"),
            Self::UpstreamTransient(message) => write!(f, "{message}"),
            Self::Network(message) => write!(f, "{message}"),
        }
    }
}

impl ProviderError {
    pub fn routing_disposition(&self) -> ProviderRoutingDisposition {
        match self {
            Self::RateLimited => ProviderRoutingDisposition::RetryNextAccount,
            Self::Unauthorized => ProviderRoutingDisposition::ExpireAccount,
            Self::CfBlocked | Self::ProxyFailed(_) => ProviderRoutingDisposition::DeactivateProxy,
            Self::UpstreamTransient(_) => ProviderRoutingDisposition::RetryNextAccount,
            Self::Network(_) => ProviderRoutingDisposition::DoNotRetry,
        }
    }

    pub fn usage_status(&self) -> &'static str {
        match self {
            Self::RateLimited => "rate_limited",
            Self::Unauthorized => "unauthorized",
            Self::CfBlocked => "cf_blocked",
            Self::ProxyFailed(_) => "proxy_failed",
            Self::UpstreamTransient(_) => "upstream_transient",
            Self::Network(_) => "error",
        }
    }

    pub fn should_mark_account_unhealthy(&self) -> bool {
        matches!(self, Self::RateLimited | Self::CfBlocked | Self::Network(_))
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
            ProviderError::UpstreamTransient(message) => AppError::GrokApi(message),
            ProviderError::Network(message) => AppError::GrokApi(message),
        }
    }
}
