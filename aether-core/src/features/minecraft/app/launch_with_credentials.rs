use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::{
        auth::Credentials,
        events::{EventEmitter, ProgressService},
        instance::{Instance, InstanceStorage},
        minecraft::{LaunchSettings, MinecraftDownloader, ReadMetadataStorage},
        process::{MinecraftProcessMetadata, ProcessStorage},
        settings::{Hooks, Settings, SettingsStorage},
    },
    shared::domain::AsyncUseCaseWithInputAndError,
};

use super::LaunchMinecraftUseCase;

pub struct LaunchWithCredentialsUseCase<
    IS: InstanceStorage,
    MS: ReadMetadataStorage,
    PS: ProcessStorage,
    SS: SettingsStorage,
    E: EventEmitter,
    MD: MinecraftDownloader,
    PGS: ProgressService,
> {
    instance_storage: Arc<IS>,
    settings_storage: Arc<SS>,
    launch_minecraft_use_case: LaunchMinecraftUseCase<IS, MS, PS, E, MD, PGS>,
}

impl<
        IS: InstanceStorage,
        MS: ReadMetadataStorage,
        PS: ProcessStorage,
        SS: SettingsStorage,
        E: EventEmitter,
        MD: MinecraftDownloader,
        PGS: ProgressService,
    > LaunchWithCredentialsUseCase<IS, MS, PS, SS, E, MD, PGS>
{
    pub fn new(
        instance_storage: Arc<IS>,
        settings_storage: Arc<SS>,
        launch_minecraft_use_case: LaunchMinecraftUseCase<IS, MS, PS, E, MD, PGS>,
    ) -> Self {
        Self {
            instance_storage,
            launch_minecraft_use_case,
            settings_storage,
        }
    }

    fn resolve_launch_settings(instance: &Instance, settings: &Settings) -> LaunchSettings {
        LaunchSettings {
            extra_launch_args: instance
                .extra_launch_args
                .clone()
                .unwrap_or_else(|| settings.extra_launch_args.clone()),

            custom_env_vars: instance
                .custom_env_vars
                .clone()
                .unwrap_or_else(|| settings.custom_env_vars.clone()),

            memory: instance.memory.unwrap_or(settings.memory),

            game_resolution: instance.game_resolution.unwrap_or(settings.game_resolution),

            hooks: Hooks {
                pre_launch: instance
                    .hooks
                    .pre_launch
                    .clone()
                    .or_else(|| settings.hooks.pre_launch.clone()),

                wrapper: instance
                    .hooks
                    .wrapper
                    .clone()
                    .or_else(|| settings.hooks.wrapper.clone()),

                post_exit: instance
                    .hooks
                    .post_exit
                    .clone()
                    .or_else(|| settings.hooks.post_exit.clone()),
            },
        }
    }
}

#[async_trait]
impl<
        IS: InstanceStorage,
        MS: ReadMetadataStorage,
        PS: ProcessStorage,
        SS: SettingsStorage,
        E: EventEmitter,
        MD: MinecraftDownloader,
        PGS: ProgressService,
    > AsyncUseCaseWithInputAndError for LaunchWithCredentialsUseCase<IS, MS, PS, SS, E, MD, PGS>
{
    type Input = (String, Credentials);
    type Output = MinecraftProcessMetadata;
    type Error = crate::Error;

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let (instance_id, credentials) = input;

        let settings = self.settings_storage.get().await?;
        let instance = self.instance_storage.get(&instance_id).await?;

        let launch_settings = Self::resolve_launch_settings(&instance, &settings);

        self.launch_minecraft_use_case
            .execute((instance_id, launch_settings, credentials))
            .await
    }
}
