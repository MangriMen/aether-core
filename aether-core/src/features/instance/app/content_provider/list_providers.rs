use std::{collections::HashMap, sync::Arc};

use crate::features::instance::{ContentProvider, ContentProviderRegistry, InstanceError};

pub struct ListProvidersUseCase<CP> {
    provider_registry: Arc<ContentProviderRegistry<CP>>,
}

impl<CP: ContentProvider> ListProvidersUseCase<CP> {
    pub fn new(provider_registry: Arc<ContentProviderRegistry<CP>>) -> Self {
        Self { provider_registry }
    }

    pub async fn execute(&self) -> Result<HashMap<String, String>, InstanceError> {
        Ok(self.provider_registry.list())
    }
}
