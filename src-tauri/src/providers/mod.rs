pub mod claude;

use async_trait::async_trait;

use crate::domain::{ProviderId, UsageWindow};
use crate::error::ProviderError;

#[async_trait]
pub trait UsageProvider: Send + Sync {
    fn id(&self) -> ProviderId;
    async fn fetch(&self) -> Result<Vec<UsageWindow>, ProviderError>;
}
