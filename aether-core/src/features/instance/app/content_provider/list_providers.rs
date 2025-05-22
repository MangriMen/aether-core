use std::{collections::HashMap, sync::Arc};

use crate::features::instance::{ContentProvider, ContentProviderRegistry};

pub struct ListProvidersUseCase<CP> {
    provider_registry: Arc<ContentProviderRegistry<CP>>,
}

impl<CP: ContentProvider> ListProvidersUseCase<CP> {
    pub fn new(provider_registry: Arc<ContentProviderRegistry<CP>>) -> Self {
        Self { provider_registry }
    }

    pub async fn execute(&self) -> crate::Result<HashMap<String, String>> {
        Ok(self.provider_registry.list())
    }
}
