use async_trait::async_trait;

use crate::account::types::GrokCookies;
use crate::grok::imagine_ws;
use crate::providers::image_provider::ImageProvider;
use crate::providers::types::{GeneratedAsset, ProviderError};

#[derive(Clone, Default)]
pub struct GrokImageProvider;

#[async_trait]
impl ImageProvider for GrokImageProvider {
    async fn generate_images(
        &self,
        cookies: &GrokCookies,
        proxy_url: Option<&String>,
        prompt: &str,
        model: &str,
    ) -> Result<Vec<GeneratedAsset>, ProviderError> {
        let enable_pro = model.to_ascii_lowercase().contains("pro");
        let assets = imagine_ws::generate_images(cookies, prompt, enable_pro, proxy_url)
            .await
            .map_err(ProviderError::from)?;

        Ok(assets.into_iter().map(GeneratedAsset::from).collect())
    }
}
