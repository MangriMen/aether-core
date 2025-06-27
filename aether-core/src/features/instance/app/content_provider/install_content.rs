use std::sync::Arc;

use crate::features::instance::{
    ContentInstallParams, ContentProvider, ContentProviderRegistry, InstanceError, PackFile,
    PackStorage,
};

pub struct InstallContentUseCase<PS: PackStorage, CP: ContentProvider> {
    pack_storage: Arc<PS>,
    provider_registry: Arc<ContentProviderRegistry<CP>>,
}

impl<PS: PackStorage, CP: ContentProvider> InstallContentUseCase<PS, CP> {
    pub fn new(pack_storage: Arc<PS>, provider_registry: Arc<ContentProviderRegistry<CP>>) -> Self {
        Self {
            pack_storage,
            provider_registry,
        }
    }

    pub async fn execute(
        &self,
        instance_id: String,
        install_params: ContentInstallParams,
    ) -> Result<(), InstanceError> {
        let provider = self
            .provider_registry
            .get(&install_params.provider.to_string())?;

        let instance_file = provider.install(&instance_id, &install_params).await?;

        self.pack_storage
            .update_pack_file(
                &instance_id,
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
}
