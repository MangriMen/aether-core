use std::collections::HashMap;

use crate::features::instance::{
    ContentInstallParams, ContentProvider, ContentSearchParams, ContentSearchResult, PackFile,
    PackStorage,
};

pub struct ContentProviderService<PS, CP>
where
    PS: PackStorage + Send + Sync,
    CP: ContentProvider + Send + Sync,
{
    pack_storage: PS,
    content_providers: HashMap<String, CP>,
}

impl<PS, CP> ContentProviderService<PS, CP>
where
    PS: PackStorage + Send + Sync,
    CP: ContentProvider + Send + Sync,
{
    pub fn new(pack_storage: PS, content_providers: HashMap<String, CP>) -> Self {
        Self {
            pack_storage,
            content_providers,
        }
    }

    pub async fn list_providers(&self) -> crate::Result<HashMap<String, String>> {
        Ok(self
            .content_providers
            .keys()
            .map(|key| (key.to_uppercase(), key.to_string()))
            .collect())
    }

    pub async fn search(
        &self,
        search_params: &ContentSearchParams,
    ) -> crate::Result<ContentSearchResult> {
        let provider = self.get_provider(&search_params.provider.to_string())?;
        provider.search(search_params).await
    }

    pub async fn install(
        &self,
        instance_id: &str,
        install_params: &ContentInstallParams,
    ) -> crate::Result<()> {
        let provider = self.get_provider(&install_params.provider.to_string())?;

        let instance_file = provider.install(instance_id, install_params).await?;

        self.pack_storage
            .update_pack_file(
                instance_id,
                &instance_file.path,
                &PackFile {
                    name: instance_file.name.clone(),
                    file_name: instance_file.file_name.clone(),
                    hash: instance_file.hash,
                    download: None,
                    option: None,
                    side: None,
                    update_provider: Some(install_params.provider.to_owned()),
                    update: instance_file.update,
                },
            )
            .await?;

        Ok(())
    }

    pub fn get_update_data_id_field(&self, provider_id: &str) -> crate::Result<String> {
        let provider = self.get_provider(provider_id)?;
        Ok(provider.get_update_data_id_field())
    }

    fn get_provider(&self, provider_id: &str) -> crate::Result<&CP> {
        self.content_providers.get(provider_id).ok_or_else(|| {
            crate::ErrorKind::ContentProviderNotFound {
                provider: provider_id.to_string(),
            }
            .as_error()
        })
    }
}
