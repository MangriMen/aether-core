use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::{
        instance::{Instance, InstanceInstallStage, InstanceStorage},
        minecraft::{InstallMinecraftUseCase, ReadMetadataStorage},
    },
    shared::domain::AsyncUseCaseWithInputAndError,
};

pub struct InstallInstanceUseCase<IS: InstanceStorage, MS: ReadMetadataStorage> {
    instance_storage: Arc<IS>,
    install_minecraft_use_case: Arc<InstallMinecraftUseCase<IS, MS>>,
}

impl<IS: InstanceStorage, MS: ReadMetadataStorage> InstallInstanceUseCase<IS, MS> {
    pub fn new(
        instance_storage: Arc<IS>,
        install_minecraft_use_case: Arc<InstallMinecraftUseCase<IS, MS>>,
    ) -> Self {
        Self {
            instance_storage,
            install_minecraft_use_case,
        }
    }

    async fn handle_failed_installation(&self, instance: &mut Instance) -> crate::Result<()> {
        if instance.install_stage != InstanceInstallStage::Installed {
            instance.install_stage = InstanceInstallStage::NotInstalled;
            self.instance_storage.upsert(instance).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl<IS, MS> AsyncUseCaseWithInputAndError for InstallInstanceUseCase<IS, MS>
where
    IS: InstanceStorage + Send + Sync,
    MS: ReadMetadataStorage + Send + Sync,
{
    type Input = (String, bool);
    type Output = ();
    type Error = crate::Error;

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let (instance_id, force) = input;

        let mut instance = self.instance_storage.get(&instance_id).await?;

        if self
            .install_minecraft_use_case
            .execute((instance_id.clone(), None, force))
            .await
            .is_err()
        {
            self.handle_failed_installation(&mut instance).await?;
        }

        Ok(())
    }
}
