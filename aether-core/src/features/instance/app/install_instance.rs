use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::{
        instance::{Instance, InstanceInstallStage, InstanceStorage},
        minecraft::{self, LoaderVersionResolver, ReadMetadataStorage},
    },
    shared::domain::AsyncUseCaseWithInputAndError,
};

pub struct InstallInstanceUseCase<IS, MS: ReadMetadataStorage> {
    instance_storage: Arc<IS>,
    loader_version_resolver: Arc<LoaderVersionResolver<MS>>,
}

impl<IS: InstanceStorage, MS: ReadMetadataStorage> InstallInstanceUseCase<IS, MS> {
    pub fn new(
        instance_storage: Arc<IS>,
        loader_version_resolver: Arc<LoaderVersionResolver<MS>>,
    ) -> Self {
        Self {
            instance_storage,
            loader_version_resolver,
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
        let mut instance = self.instance_storage.get(&input.0).await?;

        if minecraft::install_minecraft(
            self.instance_storage.clone(),
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
