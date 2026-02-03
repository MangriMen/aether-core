use std::sync::Arc;

use crate::{
    features::instance::{
        ContentProvider, ContentSearchParams, ContentSearchResult, InstanceError,
    },
    shared::CapabilityRegistry,
};

pub struct SearchContentUseCase<CP: CapabilityRegistry<Arc<dyn ContentProvider>>> {
    provider_registry: Arc<CP>,
}

impl<CP: CapabilityRegistry<Arc<dyn ContentProvider>>> SearchContentUseCase<CP> {
    pub fn new(provider_registry: Arc<CP>) -> Self {
        Self { provider_registry }
    }

    pub async fn execute(
        &self,
        search_params: ContentSearchParams,
    ) -> Result<ContentSearchResult, InstanceError> {
        let providers = self
            .provider_registry
            .find_by_capability_id(&search_params.provider)
            .await
            .map_err(|_| InstanceError::ContentProviderNotFound {
                provider_id: search_params.provider.to_string(),
            })?;

        let provider = providers
            .first()
            .ok_or(InstanceError::ContentProviderNotFound {
                provider_id: search_params.provider.to_string(),
            })?;

        provider.capability.search(&search_params).await
    }
}
