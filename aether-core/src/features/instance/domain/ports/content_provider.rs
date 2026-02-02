use async_trait::async_trait;

use crate::features::instance::{
    ContentFile, ContentInstallParams, ContentProviderCapability, ContentSearchParams,
    ContentSearchResult, InstanceError,
};

#[async_trait]
pub trait ContentProvider: Send + Sync {
    fn info(&self) -> &ContentProviderCapability;

    fn get_name(&self) -> String;

    async fn search(
        &self,
        search_content: &ContentSearchParams,
    ) -> Result<ContentSearchResult, InstanceError>;

    async fn install(
        &self,
        instance_id: &str,
        install_params: &ContentInstallParams,
    ) -> Result<ContentFile, InstanceError>;

    fn get_update_data_id_field(&self) -> String;
}
