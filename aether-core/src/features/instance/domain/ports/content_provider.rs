use async_trait::async_trait;

use crate::features::instance::{
    ContentInstallParams, ContentSearchParams, ContentSearchResult, InstanceFile,
};

#[async_trait]
pub trait ContentProvider {
    async fn search(
        &self,
        search_content: &ContentSearchParams,
    ) -> crate::Result<ContentSearchResult>;

    async fn install(
        &self,
        instance_id: &str,
        install_params: &ContentInstallParams,
    ) -> crate::Result<InstanceFile>;

    fn get_update_data_id_field(&self) -> String;
}
