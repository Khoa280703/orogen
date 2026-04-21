use async_trait::async_trait;

use crate::account::pool::CurrentAccount;
use crate::providers::types::{GeneratedAsset, ProviderError};

#[async_trait]
pub trait ImageProvider: Send + Sync {
    async fn generate_images(
        &self,
        account: &CurrentAccount,
        prompt: &str,
        model: &str,
    ) -> Result<Vec<GeneratedAsset>, ProviderError>;
}
