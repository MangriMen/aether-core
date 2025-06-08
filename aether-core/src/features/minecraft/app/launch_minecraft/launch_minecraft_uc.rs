use std::{path::Path, sync::Arc};

use crate::{
    core::{domain::LazyLocator, LauncherState},
    features::{
        auth::Credentials,
        events::{EventEmitter, ProgressService},
        instance::{InstanceInstallStage, InstanceStorage, InstanceStorageExt},
        java::{GetJavaUseCase, JavaInstallationService, JavaStorage},
        minecraft::{
            get_compatible_java_version, resolve_minecraft_version, GetVersionManifestUseCase,
            InstallMinecraftUseCase, LaunchSettings, LoaderVersionResolver, MinecraftDownloader,
            ModLoader, ReadMetadataStorage,
        },
        plugins::PluginEvent,
        process::{
            GetProcessMetadataByInstanceIdUseCase, MinecraftProcessMetadata, ProcessStorage,
            StartProcessUseCase,
        },
    },
    libs::request_client::RequestClient,
    shared::{IOError, SerializableCommand},
    with_mut_ref,
};

use super::{
    get_minecraft_arguments::get_minecraft_arguments,
    get_minecraft_jvm_arguments::get_minecraft_jvm_arguments,
};

pub struct LaunchMinecraftUseCase<
    IS: InstanceStorage,
    MS: ReadMetadataStorage,
    PS: ProcessStorage,
    E: EventEmitter,
    MD: MinecraftDownloader,
    PGS: ProgressService,
    JIS: JavaInstallationService,
    JS: JavaStorage,
    RC: RequestClient,
> {
    instance_storage: Arc<IS>,
    loader_version_resolver: Arc<LoaderVersionResolver<MS>>,
    install_minecraft_use_case: Arc<InstallMinecraftUseCase<IS, MS, MD, PGS, JIS, JS, RC>>,
    get_version_manifest_use_case: Arc<GetVersionManifestUseCase<MS>>,
    get_process_by_instance_id_use_case: Arc<GetProcessMetadataByInstanceIdUseCase<PS>>,
    start_process_use_case: Arc<StartProcessUseCase<E, PS>>,
    minecraft_downloader: MD,
    java_installation_service: JIS,
    get_java_use_case: Arc<GetJavaUseCase<JS, JIS>>,
}

impl<
        IS: InstanceStorage,
        MS: ReadMetadataStorage,
        PS: ProcessStorage,
        E: EventEmitter,
        MD: MinecraftDownloader,
        PGS: ProgressService,
        JIS: JavaInstallationService,
        JS: JavaStorage,
        RC: RequestClient,
    > LaunchMinecraftUseCase<IS, MS, PS, E, MD, PGS, JIS, JS, RC>
{
    pub fn new(
        instance_storage: Arc<IS>,
        loader_version_resolver: Arc<LoaderVersionResolver<MS>>,
        install_minecraft_use_case: Arc<InstallMinecraftUseCase<IS, MS, MD, PGS, JIS, JS, RC>>,
        get_version_manifest_use_case: Arc<GetVersionManifestUseCase<MS>>,
        get_process_by_instance_id_use_case: Arc<GetProcessMetadataByInstanceIdUseCase<PS>>,
        start_process_use_case: Arc<StartProcessUseCase<E, PS>>,
        minecraft_downloader: MD,
        java_installation_service: JIS,
        get_java_use_case: Arc<GetJavaUseCase<JS, JIS>>,
    ) -> Self {
        Self {
            instance_storage,
            loader_version_resolver,
            install_minecraft_use_case,
            get_version_manifest_use_case,
            get_process_by_instance_id_use_case,
            start_process_use_case,
            minecraft_downloader,
            java_installation_service,
            get_java_use_case,
        }
    }

    pub async fn execute(
        &self,
        instance_id: String,
        launch_settings: LaunchSettings,
        credentials: Credentials,
    ) -> crate::Result<MinecraftProcessMetadata> {
        let instance = self.instance_storage.get(&instance_id).await?;

        if instance.install_stage == InstanceInstallStage::PackInstalling
            || instance.install_stage == InstanceInstallStage::Installing
        {
            return Err(crate::ErrorKind::LauncherError(
                "Instance is still installing".to_string(),
            )
            .into());
        }

        if instance.install_stage != InstanceInstallStage::Installed {
            self.install_minecraft_use_case
                .execute(instance_id, None, false)
                .await?;
        }

        let state = LauncherState::get().await?;

        let pre_launch_command = instance
            .hooks
            .pre_launch
            .as_ref()
            .or(launch_settings.hooks.pre_launch.as_ref());

        let instance_path = state.location_info.instance_dir(&instance.id);

        run_pre_launch_command(&pre_launch_command, &instance_path).await?;

        let lazy_locator = LazyLocator::get().await?;
        let plugin_registry = lazy_locator.get_plugin_registry().await;

        if let Some(pack_info) = &instance.pack_info {
            if let Ok(plugin) = plugin_registry.get(&pack_info.pack_type) {
                if let Some(plugin) = &plugin.instance {
                    let mut plugin = plugin.lock().await;
                    if plugin.supports_handle_events() {
                        plugin.handle_event(&PluginEvent::BeforeInstanceLaunch {
                            instance_id: instance.id.clone(),
                        })?;
                    }
                }
            }
        }

        let version_manifest = self.get_version_manifest_use_case.execute().await?;

        let (version, minecraft_updated) =
            resolve_minecraft_version(&instance.game_version, version_manifest)?;

        let loader_version = self
            .loader_version_resolver
            .resolve(
                &instance.game_version,
                &instance.loader,
                &instance.loader_version,
            )
            .await?;

        if instance.loader != ModLoader::Vanilla && loader_version.is_none() {
            return Err(crate::ErrorKind::LauncherError(format!(
                "No loader version selected for {}",
                instance.loader.as_str()
            ))
            .into());
        }

        let version_jar = loader_version.as_ref().map_or(version.id.clone(), |it| {
            format!("{}-{}", version.id.clone(), it.id.clone())
        });

        let version_info = self
            .minecraft_downloader
            .download_version_info(&version, loader_version.as_ref(), None, None)
            .await?;

        let java = if let Some(java_path) = instance.java_path.as_ref() {
            self.java_installation_service
                .locate_java(Path::new(java_path))
                .await
        } else {
            let compatible_java_version = get_compatible_java_version(&version_info);
            self.get_java_use_case
                .execute(compatible_java_version)
                .await
        }?;

        let client_path = state
            .location_info
            .version_dir(&version_jar)
            .join(format!("{version_jar}.jar"));

        let args = version_info.arguments.clone().unwrap_or_default();

        let env_args_vec = launch_settings.custom_env_vars.clone();

        let mut command = match &launch_settings.hooks.wrapper {
            Some(hook) => {
                with_mut_ref!(it = tokio::process::Command::new(hook) => {it.arg(&java.path)})
            }
            None => tokio::process::Command::new(&java.path),
        };

        // Check if profile has a running profile, and reject running the command if it does
        // Done late so a quick double call doesn't launch two instances
        let existing_processes = self
            .get_process_by_instance_id_use_case
            .execute(instance.id.clone())
            .await;
        if let Some(process) = existing_processes.first() {
            return Err(crate::ErrorKind::LauncherError(format!(
                "Profile {} is already running at path: {}",
                instance.id, process.uuid
            ))
            .as_error());
        }

        let natives_dir = state.location_info.version_natives_dir(&version_jar);
        if !natives_dir.exists() {
            tokio::fs::create_dir_all(&natives_dir).await?;
        }

        let jvm_arguments = get_minecraft_jvm_arguments(
            args.get(&daedalus::minecraft::ArgumentType::Jvm)
                .map(|x| x.as_slice()),
            &state.location_info.libraries_dir(),
            &version_info,
            &natives_dir,
            &client_path,
            version_jar,
            &java,
            launch_settings.memory,
            &launch_settings.extra_launch_args,
            minecraft_updated,
        )?;

        let minecraft_arguments = get_minecraft_arguments(
            args.get(&daedalus::minecraft::ArgumentType::Game)
                .map(|x| x.as_slice()),
            version_info.minecraft_arguments.as_deref(),
            &credentials,
            &version.id,
            &version_info.asset_index.id,
            &instance_path,
            &state.location_info.assets_dir(),
            &version.type_,
            launch_settings.game_resolution,
            &java.architecture,
        )?
        .into_iter()
        .collect::<Vec<_>>();

        command
            .args(jvm_arguments)
            .arg(version_info.main_class.clone())
            .args(minecraft_arguments)
            .current_dir(instance_path.clone());

        // CARGO-set DYLD_LIBRARY_PATH breaks Minecraft on macOS during testing on playground
        #[cfg(target_os = "macos")]
        if std::env::var("CARGO").is_ok() {
            command.env_remove("DYLD_FALLBACK_LIBRARY_PATH");
        }

        // Java options should be set in instance options (the existence of _JAVA_OPTIONS overwrites them)
        command.env_remove("_JAVA_OPTIONS");

        command.envs(env_args_vec);

        // options.txt override

        // authentication credentials

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

        if let Some(pack_info) = &instance.pack_info {
            if let Ok(plugin) = plugin_registry.get(&pack_info.pack_type) {
                if let Some(plugin) = &plugin.instance {
                    let mut plugin = plugin.lock().await;
                    if plugin.supports_handle_events() {
                        plugin.handle_event(&PluginEvent::AfterInstanceLaunch {
                            instance_id: instance.id.clone(),
                        })?;
                    }
                }
            }
        }

        metadata
    }
}

async fn run_pre_launch_command(
    pre_launch_command: &Option<&String>,
    working_dir: &Path,
) -> crate::Result<()> {
    if let Some(command) = pre_launch_command {
        if let Ok(cmd) = SerializableCommand::from_string(command, Some(&working_dir.to_path_buf()))
        {
            let result = cmd
                .to_tokio_command()
                .spawn()
                .map_err(|e| IOError::with_path(e, working_dir))?
                .wait()
                .await
                .map_err(IOError::from)?;

            if !result.success() {
                return Err(crate::ErrorKind::LauncherError(format!(
                    "Non-zero exit code for pre-launch hook: {}",
                    result.code().unwrap_or(-1)
                ))
                .as_error());
            }
        }
    }

    Ok(())
}
