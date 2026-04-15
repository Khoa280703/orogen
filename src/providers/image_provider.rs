use async_trait::async_trait;

use crate::account::types::GrokCookies;
use crate::providers::types::{GeneratedAsset, ProviderError};

#[async_trait]
pub trait ImageProvider: Send + Sync {
    async fn generate_images(
        &self,
        cookies: &GrokCookies,
        proxy_url: Option<&String>,
        prompt: &str,
        model: &str,
    ) -> Result<Vec<GeneratedAsset>, ProviderError>;
}
