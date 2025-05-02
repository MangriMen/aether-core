use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::instance::{ContentProvider, ProviderRegistry},
    shared::domain::AsyncUseCaseWithInputAndError,
};

pub struct GetProviderMetadataUseCase<CP: ContentProvider> {
    provider_registry: Arc<ProviderRegistry<CP>>,
}

impl<CP: ContentProvider> GetProviderMetadataUseCase<CP> {
    pub fn new(provider_registry: Arc<ProviderRegistry<CP>>) -> Self {
        Self { provider_registry }
    }
}

#[async_trait]
impl<CP> AsyncUseCaseWithInputAndError for GetProviderMetadataUseCase<CP>
where
    CP: ContentProvider + Send + Sync,
{
    type Input = String;
    type Output = String;
    type Error = crate::Error;

    async fn execute(&self, provider_id: Self::Input) -> Result<Self::Output, Self::Error> {
        let provider = self.provider_registry.get(&provider_id)?;
        Ok(provider.get_update_data_id_field())
    }
}
