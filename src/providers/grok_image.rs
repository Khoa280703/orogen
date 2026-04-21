use async_trait::async_trait;

use crate::account::pool::CurrentAccount;
use crate::grok::imagine_ws;
use crate::providers::image_provider::ImageProvider;
use crate::providers::types::{GeneratedAsset, ProviderError};

#[derive(Clone, Default)]
pub struct GrokImageProvider;

#[async_trait]
impl ImageProvider for GrokImageProvider {
    async fn generate_images(
        &self,
        account: &CurrentAccount,
        prompt: &str,
        model: &str,
    ) -> Result<Vec<GeneratedAsset>, ProviderError> {
        let cookies = account.grok_cookies().map_err(ProviderError::Network)?;
        let enable_pro = model.to_ascii_lowercase().contains("pro");
        let assets =
            imagine_ws::generate_images(cookies, prompt, enable_pro, account.proxy_url.as_ref())
                .await
                .map_err(ProviderError::from)?;

        Ok(assets.into_iter().map(GeneratedAsset::from).collect())
    }
}
