use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::instance::{
        ContentProvider, ContentSearchParams, ContentSearchResult, ProviderRegistry,
    },
    shared::domain::AsyncUseCaseWithInputAndError,
};

pub struct SearchContentUseCase<CP: ContentProvider> {
    provider_registry: Arc<ProviderRegistry<CP>>,
}

impl<CP: ContentProvider> SearchContentUseCase<CP> {
    pub fn new(provider_registry: Arc<ProviderRegistry<CP>>) -> Self {
        Self { provider_registry }
    }
}

#[async_trait]
impl<CP> AsyncUseCaseWithInputAndError for SearchContentUseCase<CP>
where
    CP: ContentProvider + Send + Sync,
{
    type Input = ContentSearchParams;
    type Output = ContentSearchResult;
    type Error = crate::Error;

    async fn execute(&self, search_params: Self::Input) -> Result<Self::Output, Self::Error> {
        let provider = self
            .provider_registry
            .get(&search_params.provider.to_string())?;
        provider.search(&search_params).await
    }
}
