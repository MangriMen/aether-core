use std::sync::Arc;

use crate::{
    features::{
        events::ProgressService,
        instance::{Instance, InstanceInstallStage, InstanceStorage},
        java::{JavaInstallationService, JavaStorage},
        minecraft::{InstallMinecraftUseCase, MinecraftDownloader, ReadMetadataStorage},
    },
    libs::request_client::RequestClient,
};

pub struct InstallInstanceUseCase<
    IS: InstanceStorage,
    MS: ReadMetadataStorage,
    MD: MinecraftDownloader,
    PS: ProgressService,
    JIS: JavaInstallationService,
    JS: JavaStorage,
    RC: RequestClient,
> {
    instance_storage: Arc<IS>,
    install_minecraft_use_case: Arc<InstallMinecraftUseCase<IS, MS, MD, PS, JIS, JS, RC>>,
}

impl<
        IS: InstanceStorage,
        MS: ReadMetadataStorage,
        MD: MinecraftDownloader,
        PS: ProgressService,
        JIS: JavaInstallationService,
        JS: JavaStorage,
        RC: RequestClient,
    > InstallInstanceUseCase<IS, MS, MD, PS, JIS, JS, RC>
{
    pub fn new(
        instance_storage: Arc<IS>,
        install_minecraft_use_case: Arc<InstallMinecraftUseCase<IS, MS, MD, PS, JIS, JS, RC>>,
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

    pub async fn execute(&self, instance_id: String, force: bool) -> crate::Result<()> {
        let mut instance = self.instance_storage.get(&instance_id).await?;

        if self
            .install_minecraft_use_case
            .execute(instance_id.clone(), None, force)
            .await
            .is_err()
        {
            self.handle_failed_installation(&mut instance).await?;
        }

        Ok(())
    }
}
