use async_trait::async_trait;

use crate::features::instance::{
    ContentInstallParams, ContentSearchParams, ContentSearchResult, InstanceError, InstanceFile,
};

#[async_trait]
pub trait ContentProvider: Send + Sync {
    async fn search(
        &self,
        search_content: &ContentSearchParams,
    ) -> Result<ContentSearchResult, InstanceError>;

    async fn install(
        &self,
        instance_id: &str,
        install_params: &ContentInstallParams,
    ) -> Result<InstanceFile, InstanceError>;

    fn get_update_data_id_field(&self) -> String;
}
