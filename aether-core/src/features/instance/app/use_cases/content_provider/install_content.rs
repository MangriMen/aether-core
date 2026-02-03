use std::sync::Arc;

use crate::{
    features::instance::{
        ContentInstallParams, ContentProvider, InstanceError, PackFile, PackStorage,
    },
    shared::CapabilityRegistry,
};

pub struct InstallContentUseCase<PS: PackStorage, CP: CapabilityRegistry<Arc<dyn ContentProvider>>>
{
    pack_storage: Arc<PS>,
    provider_registry: Arc<CP>,
}

impl<PS: PackStorage, CP: CapabilityRegistry<Arc<dyn ContentProvider>>>
    InstallContentUseCase<PS, CP>
{
    pub fn new(pack_storage: Arc<PS>, provider_registry: Arc<CP>) -> Self {
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
        let providers = self
            .provider_registry
            .find_by_capability_id(&install_params.provider)
            .await
            .map_err(|_| InstanceError::ContentProviderNotFound {
                provider_id: install_params.provider.to_string(),
            })?;

        let provider = providers
            .first()
            .ok_or(InstanceError::ContentProviderNotFound {
                provider_id: install_params.provider.to_string(),
            })?;

        let instance_file = provider
            .capability
            .install(&instance_id, &install_params)
            .await?;

        self.pack_storage
            .update_pack_file(
                &instance_id,
                &instance_file.instance_relative_path,
                &PackFile {
                    name: instance_file.name.clone(),
                    file_name: instance_file.filename.clone(),
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
