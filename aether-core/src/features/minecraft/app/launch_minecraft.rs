use crate::{
    api,
    core::LauncherState,
    features::{
        auth::Credentials,
        instance::{Instance, InstanceInstallStage, ModLoader},
        minecraft::{self, LaunchSettings},
        plugins::PluginEvent,
        process::{MinecraftProcessMetadata, ProcessManager},
        settings::SerializableCommand,
    },
    shared::IOError,
    with_mut_ref,
};

use super::install_minecraft;

#[tracing::instrument]
pub async fn launch_minecraft(
    instance: &Instance,
    launch_settings: &LaunchSettings,
    credentials: &Credentials,
) -> crate::Result<MinecraftProcessMetadata> {
    if instance.install_stage == InstanceInstallStage::PackInstalling
        || instance.install_stage == InstanceInstallStage::Installing
    {
        return Err(
            crate::ErrorKind::LauncherError("Instance is still installing".to_string()).into(),
        );
    }

    if instance.install_stage != InstanceInstallStage::Installed {
        install_minecraft(instance, None, false).await?;
    }

    let state = LauncherState::get().await?;

    run_pre_launch_command(instance, launch_settings).await?;

    let plugin_manager = state.plugin_manager.read().await;

    if let Some(pack_info) = &instance.pack_info {
        if let Ok(plugin) = plugin_manager.get_plugin(&pack_info.pack_type) {
            if let Some(plugin) = plugin.get_plugin() {
                let mut plugin = plugin.lock().await;
                if plugin.supports_handle_events() {
                    plugin.handle_event(&PluginEvent::BeforeInstanceLaunch {
                        instance_id: instance.id.clone(),
                    })?;
                }
            }
        }
    }

    let instance_path = Instance::get_full_path(&instance.id).await?;

    let version_manifest = api::metadata::get_version_manifest().await?;

    let (version, minecraft_updated) =
        minecraft::resolve_minecraft_version(&instance.game_version, version_manifest)?;

    let loader_version: Option<daedalus::modded::LoaderVersion> = Instance::get_loader_version(
        &instance.game_version,
        instance.loader,
        instance.loader_version.as_deref(),
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

    let version_info =
        minecraft::download_version_info(&state, &version, loader_version.as_ref(), None, None)
            .await?;

    let java = if let Some(java) = instance.get_java().await.transpose() {
        java
    } else {
        let compatible_java_version = minecraft::get_compatible_java_version(&version_info);

        crate::api::java::get(compatible_java_version).await
    }?;

    let client_path = state
        .locations
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
    let existing_processes = api::process::get_by_instance_id(&instance.id).await?;
    if let Some(process) = existing_processes.first() {
        return Err(crate::ErrorKind::LauncherError(format!(
            "Profile {} is already running at path: {}",
            instance.id, process.uuid
        ))
        .as_error());
    }

    let natives_dir = state.locations.version_natives_dir(&version_jar);
    if !natives_dir.exists() {
        tokio::fs::create_dir_all(&natives_dir).await?;
    }

    let jvm_arguments = minecraft::get_minecraft_jvm_arguments(
        args.get(&daedalus::minecraft::ArgumentType::Jvm)
            .map(|x| x.as_slice()),
        &state.locations.libraries_dir(),
        &version_info,
        &natives_dir,
        &client_path,
        version_jar,
        &java,
        launch_settings.memory,
        &launch_settings.extra_launch_args,
        minecraft_updated,
    )?;

    let minecraft_arguments = minecraft::get_minecraft_arguments(
        args.get(&daedalus::minecraft::ArgumentType::Game)
            .map(|x| x.as_slice()),
        version_info.minecraft_arguments.as_deref(),
        credentials,
        &version.id,
        &version_info.asset_index.id,
        &instance_path,
        &state.locations.assets_dir(),
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

    Instance::edit(&instance.id, |instance| {
        instance.last_played = Some(chrono::Utc::now());
        async { Ok(()) }
    })
    .await?;

    let metadata = state
        .process_manager
        .insert_new_process(
            &instance.id,
            command,
            launch_settings.hooks.post_exit.clone(),
        )
        .await;

    if let Some(pack_info) = &instance.pack_info {
        if let Ok(plugin) = plugin_manager.get_plugin(&pack_info.pack_type) {
            if let Some(plugin) = plugin.get_plugin() {
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

async fn run_pre_launch_command(
    instance: &Instance,
    launch_settings: &LaunchSettings,
) -> crate::Result<()> {
    let pre_launch_commands = instance
        .hooks
        .pre_launch
        .as_ref()
        .or(launch_settings.hooks.pre_launch.as_ref());

    if let Some(command) = pre_launch_commands {
        let full_path = &instance.path;
        if let Ok(cmd) = SerializableCommand::from_string(command, Some(full_path)) {
            let result = cmd
                .to_tokio_command()
                .spawn()
                .map_err(|e| IOError::with_path(e, full_path))?
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
