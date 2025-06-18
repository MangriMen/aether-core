use std::sync::Arc;

use crate::{
    features::{
        auth::Credentials,
        events::{EventEmitter, ProgressService},
        instance::{
            InstallInstanceUseCase, Instance, InstanceInstallStage, InstanceStorage,
            InstanceStorageExt,
        },
        java::{JavaInstallationService, JavaStorage},
        minecraft::{
            GetMinecraftLaunchCommandParams, GetMinecraftLaunchCommandUseCase, LaunchSettings,
            MinecraftDownloader, ReadMetadataStorage,
        },
        process::{
            GetProcessMetadataByInstanceIdUseCase, MinecraftProcessMetadata, ProcessStorage,
            StartProcessUseCase,
        },
        settings::{Hooks, LocationInfo, Settings, SettingsStorage},
    },
    libs::request_client::RequestClient,
    shared::{IoError, SerializableCommand},
};

pub struct LaunchInstanceUseCase<
    IS: InstanceStorage,
    MS: ReadMetadataStorage,
    PS: ProcessStorage,
    SS: SettingsStorage,
    E: EventEmitter,
    MD: MinecraftDownloader,
    PGS: ProgressService,
    JIS: JavaInstallationService,
    JS: JavaStorage,
    RC: RequestClient,
> {
    instance_storage: Arc<IS>,
    settings_storage: Arc<SS>,
    location_info: Arc<LocationInfo>,
    get_process_by_instance_id_use_case: Arc<GetProcessMetadataByInstanceIdUseCase<PS>>,
    install_instance_use_case: Arc<InstallInstanceUseCase<IS, MS, MD, PGS, JIS, JS, RC>>,
    get_minecraft_launch_command_use_case: GetMinecraftLaunchCommandUseCase<MS, MD, JIS, JS>,
    start_process_use_case: Arc<StartProcessUseCase<E, PS>>,
}

impl<
        IS: InstanceStorage,
        MS: ReadMetadataStorage,
        PS: ProcessStorage + 'static,
        SS: SettingsStorage,
        E: EventEmitter + 'static,
        MD: MinecraftDownloader,
        PGS: ProgressService,
        JIS: JavaInstallationService,
        JS: JavaStorage,
        RC: RequestClient,
    > LaunchInstanceUseCase<IS, MS, PS, SS, E, MD, PGS, JIS, JS, RC>
{
    pub fn new(
        instance_storage: Arc<IS>,
        settings_storage: Arc<SS>,
        location_info: Arc<LocationInfo>,
        get_process_by_instance_id_use_case: Arc<GetProcessMetadataByInstanceIdUseCase<PS>>,
        install_instance_use_case: Arc<InstallInstanceUseCase<IS, MS, MD, PGS, JIS, JS, RC>>,
        get_minecraft_launch_command_use_case: GetMinecraftLaunchCommandUseCase<MS, MD, JIS, JS>,
        start_process_use_case: Arc<StartProcessUseCase<E, PS>>,
    ) -> Self {
        Self {
            instance_storage,
            settings_storage,
            location_info,
            get_process_by_instance_id_use_case,
            install_instance_use_case,
            get_minecraft_launch_command_use_case,
            start_process_use_case,
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

    pub async fn execute(
        &self,
        instance_id: String,
        credentials: Credentials,
    ) -> crate::Result<MinecraftProcessMetadata> {
        let settings = self.settings_storage.get().await?;
        let instance = self.instance_storage.get(&instance_id).await?;

        let launch_settings = Self::resolve_launch_settings(&instance, &settings);

        let instance = self.instance_storage.get(&instance_id).await?;

        if instance.install_stage == InstanceInstallStage::PackInstalling
            || instance.install_stage == InstanceInstallStage::Installing
        {
            return Err(crate::ErrorKind::LauncherError(
                "Instance is still installing".to_string(),
            )
            .into());
        }

        // Check if profile has a running profile, and reject running the command if it does
        // Done late so a quick double call doesn't launch two instances

        if let Some(process) = self
            .get_process_by_instance_id_use_case
            .execute(instance.id.clone())
            .await
            .first()
        {
            return Err(crate::ErrorKind::LauncherError(format!(
                "Profile {} is already running at path: {}",
                instance.id, process.uuid
            ))
            .as_error());
        }

        if instance.install_stage != InstanceInstallStage::Installed {
            self.install_instance_use_case
                .execute(instance_id, false)
                .await?;
        }

        let pre_launch_command = instance
            .hooks
            .pre_launch
            .as_ref()
            .or(launch_settings.hooks.pre_launch.as_ref());

        let instance_path = self.location_info.instance_dir(&instance.id);

        if let Some(command) = pre_launch_command {
            if let Ok(cmd) = SerializableCommand::from_string(command, Some(&instance_path)) {
                if !cmd
                    .to_tokio_command()
                    .spawn()
                    .map_err(|e| IoError::with_path(e, &instance_path))?
                    .wait()
                    .await
                    .map_err(IoError::from)?
                    .success()
                {
                    return Err(crate::ErrorKind::LauncherError(
                        "Pre launch command error".to_string(),
                    )
                    .as_error());
                    // return Err(MinecraftError::PreLaunchCommandError {
                    //     code: result.code().unwrap_or(-1),
                    // });
                }
            }
        }

        // run_pre_launch_command(&pre_launch_command, &instance_path).await?;

        // let lazy_locator = LazyLocator::get().await?;
        // let plugin_registry = lazy_locator.get_plugin_registry().await;

        // if let Some(pack_info) = &instance.pack_info {
        //     if let Ok(plugin) = plugin_registry.get(&pack_info.pack_type) {
        //         if let Some(plugin) = &plugin.instance {
        //             let mut plugin = plugin.lock().await;
        //             if plugin.supports_handle_events() {
        //                 plugin.handle_event(&PluginEvent::BeforeInstanceLaunch {
        //                     instance_id: instance.id.clone(),
        //                 })?;
        //             }
        //         }
        //     }
        // }

        let command = self
            .get_minecraft_launch_command_use_case
            .execute(
                GetMinecraftLaunchCommandParams {
                    game_version: instance.game_version.clone(),
                    loader: instance.loader,
                    loader_version: instance.loader_version.clone(),
                    launch_dir: instance_path,
                    java_path: instance.java_path.clone(),
                },
                launch_settings.clone(),
                credentials,
            )
            .await?;

        self.instance_storage
            .upsert_with(&instance.id, |instance| {
                instance.last_played = Some(chrono::Utc::now());
                Ok(())
            })
            .await?;

        let metadata = self
            .start_process_use_case
            .execute(
                instance.id.clone(),
                command,
                launch_settings.hooks.post_exit.clone(),
            )
            .await;

        // if let Some(pack_info) = &instance.pack_info {
        //     if let Ok(plugin) = plugin_registry.get(&pack_info.pack_type) {
        //         if let Some(plugin) = &plugin.instance {
        //             let mut plugin = plugin.lock().await;
        //             if plugin.supports_handle_events() {
        //                 plugin.handle_event(&PluginEvent::AfterInstanceLaunch {
        //                     instance_id: instance.id.clone(),
        //                 })?;
        //             }
        //         }
        //     }
        // }

        Ok(metadata?)
    }
}
