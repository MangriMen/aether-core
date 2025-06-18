use std::sync::Arc;

use crate::features::instance::{
    ContentProvider, ContentProviderRegistry, ContentSearchParams, ContentSearchResult,
    InstanceError,
};

pub struct SearchContentUseCase<CP: ContentProvider> {
    provider_registry: Arc<ContentProviderRegistry<CP>>,
}

impl<CP: ContentProvider> SearchContentUseCase<CP> {
    pub fn new(provider_registry: Arc<ContentProviderRegistry<CP>>) -> Self {
        Self { provider_registry }
    }

    pub async fn execute(
        &self,
        search_params: ContentSearchParams,
    ) -> Result<ContentSearchResult, InstanceError> {
        let provider = self
            .provider_registry
            .get(&search_params.provider.to_string())?;
        provider.search(&search_params).await
    }
}
