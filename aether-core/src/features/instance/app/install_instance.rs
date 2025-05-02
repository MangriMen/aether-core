use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::{
        instance::{Instance, InstanceInstallStage, InstanceManager},
        minecraft::{self, LoaderVersionResolver, ReadMetadataStorage},
    },
    shared::domain::AsyncUseCaseWithInputAndError,
};

pub struct InstallInstanceUseCase<IM: InstanceManager, MS: ReadMetadataStorage> {
    instance_manager: Arc<IM>,
    loader_version_resolver: Arc<LoaderVersionResolver<MS>>,
}

impl<IM: InstanceManager, MS: ReadMetadataStorage> InstallInstanceUseCase<IM, MS> {
    pub fn new(
        instance_manager: Arc<IM>,
        loader_version_resolver: Arc<LoaderVersionResolver<MS>>,
    ) -> Self {
        Self {
            instance_manager,
            loader_version_resolver,
        }
    }

    async fn handle_failed_installation(&self, instance: &mut Instance) -> crate::Result<()> {
        if instance.install_stage != InstanceInstallStage::Installed {
            instance.install_stage = InstanceInstallStage::NotInstalled;
            self.instance_manager.upsert(instance).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl<IM, MS> AsyncUseCaseWithInputAndError for InstallInstanceUseCase<IM, MS>
where
    IM: InstanceManager + Send + Sync,
    MS: ReadMetadataStorage + Send + Sync,
{
    type Input = (String, bool);
    type Output = ();
    type Error = crate::Error;

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let mut instance = self.instance_manager.get(&input.0).await?;

        if minecraft::install_minecraft(
            &*self.instance_manager,
            &*self.loader_version_resolver,
            &instance,
            None,
            input.1,
        )
        .await
        .is_err()
        {
            self.handle_failed_installation(&mut instance).await?;
        }

        Ok(())
    }
}
