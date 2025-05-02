use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::{
        auth::Credentials,
        instance::{resolve_launch_settings, InstanceStorage},
        minecraft::ReadMetadataStorage,
        process::{MinecraftProcessMetadata, ProcessStorage},
        settings::SettingsStorage,
    },
    shared::domain::AsyncUseCaseWithInputAndError,
};

use super::LaunchMinecraftUseCase;

pub struct LaunchWithCredentialsUseCase<
    IS: InstanceStorage,
    MS: ReadMetadataStorage,
    PS: ProcessStorage,
    SS: SettingsStorage,
> {
    instance_storage: Arc<IS>,
    settings_storage: Arc<SS>,
    launch_minecraft_use_case: LaunchMinecraftUseCase<IS, MS, PS>,
}

impl<IS: InstanceStorage, MS: ReadMetadataStorage, PS: ProcessStorage, SS: SettingsStorage>
    LaunchWithCredentialsUseCase<IS, MS, PS, SS>
{
    pub fn new(
        instance_storage: Arc<IS>,
        settings_storage: Arc<SS>,
        launch_minecraft_use_case: LaunchMinecraftUseCase<IS, MS, PS>,
    ) -> Self {
        Self {
            instance_storage,
            launch_minecraft_use_case,
            settings_storage,
        }
    }
}

#[async_trait]
impl<IS, MS, PS, SS> AsyncUseCaseWithInputAndError for LaunchWithCredentialsUseCase<IS, MS, PS, SS>
where
    IS: InstanceStorage + Send + Sync,
    MS: ReadMetadataStorage + Send + Sync,
    PS: ProcessStorage + Send + Sync,
    SS: SettingsStorage + Send + Sync,
{
    type Input = (String, Credentials);
    type Output = MinecraftProcessMetadata;
    type Error = crate::Error;

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let (instance_id, credentials) = input;

        let settings = self.settings_storage.get().await?;
        let instance = self.instance_storage.get(&instance_id).await?;

        let launch_settings = resolve_launch_settings(&instance, &settings);

        self.launch_minecraft_use_case
            .execute((instance_id, launch_settings, credentials))
            .await
    }
}
