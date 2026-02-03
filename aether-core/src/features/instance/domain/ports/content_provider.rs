use async_trait::async_trait;

use crate::features::instance::{
    ContentFile, ContentInstallParams, ContentProviderCapabilityMetadata, ContentSearchParams,
    ContentSearchResult, InstanceError,
};

#[async_trait]
pub trait ContentProvider: Send + Sync {
    fn metadata(&self) -> &ContentProviderCapabilityMetadata;

    async fn search(
        &self,
        search_content: &ContentSearchParams,
    ) -> Result<ContentSearchResult, InstanceError>;

    async fn install(
        &self,
        instance_id: &str,
        install_params: &ContentInstallParams,
    ) -> Result<ContentFile, InstanceError>;
}
