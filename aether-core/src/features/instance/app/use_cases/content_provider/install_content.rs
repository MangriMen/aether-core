use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::instance::{
        ContentInstallParams, ContentProvider, PackFile, PackStorage, ProviderRegistry,
    },
    shared::domain::AsyncUseCaseWithInputAndError,
};

pub struct InstallContentUseCase<PS: PackStorage, CP: ContentProvider> {
    pack_storage: Arc<PS>,
    provider_registry: Arc<ProviderRegistry<CP>>,
}

impl<PS: PackStorage, CP: ContentProvider> InstallContentUseCase<PS, CP> {
    pub fn new(pack_storage: Arc<PS>, provider_registry: Arc<ProviderRegistry<CP>>) -> Self {
        Self {
            pack_storage,
            provider_registry,
        }
    }
}

#[async_trait]
impl<PS, CP> AsyncUseCaseWithInputAndError for InstallContentUseCase<PS, CP>
where
    PS: PackStorage + Send + Sync,
    CP: ContentProvider + Send + Sync,
{
    type Input = (String, ContentInstallParams);
    type Output = ();
    type Error = crate::Error;

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let (instance_id, install_params) = input;

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
