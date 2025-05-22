use std::sync::Arc;

use crate::features::instance::{ContentProvider, ContentProviderRegistry};

pub struct GetProviderMetadataUseCase<CP: ContentProvider> {
    provider_registry: Arc<ContentProviderRegistry<CP>>,
}

impl<CP: ContentProvider> GetProviderMetadataUseCase<CP> {
    pub fn new(provider_registry: Arc<ContentProviderRegistry<CP>>) -> Self {
        Self { provider_registry }
    }

    pub async fn execute(&self, provider_id: String) -> crate::Result<String> {
        let provider = self.provider_registry.get(&provider_id)?;
        Ok(provider.get_update_data_id_field())
    }
}
