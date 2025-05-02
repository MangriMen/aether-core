use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;

use crate::{
    features::instance::{ContentProvider, ProviderRegistry},
    shared::domain::AsyncUseCaseWithError,
};

pub struct ListProvidersUseCase<CP> {
    provider_registry: Arc<ProviderRegistry<CP>>,
}

impl<CP> ListProvidersUseCase<CP> {
    pub fn new(provider_registry: Arc<ProviderRegistry<CP>>) -> Self {
        Self { provider_registry }
    }
}

#[async_trait]
impl<CP> AsyncUseCaseWithError for ListProvidersUseCase<CP>
where
    CP: ContentProvider + Send + Sync,
{
    type Output = HashMap<String, String>;
    type Error = crate::Error;

    async fn execute(&self) -> Result<Self::Output, Self::Error> {
        Ok(self.provider_registry.list())
    }
}
