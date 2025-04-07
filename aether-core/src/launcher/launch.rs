use daedalus::{minecraft, modded};

use crate::{
    api,
    core::LauncherState,
    event::{
        emit::{emit_loading, init_or_edit_loading},
        LoadingBarId, LoadingBarType,
    },
    features::{
        auth::Credentials,
        instance::{Instance, InstanceInstallStage, ModLoader},
        plugins::PluginEvent,
        process::MinecraftProcessMetadata,
        settings::{MemorySettings, WindowSize},
    },
    launcher::mod_loader_post_install,
    utils::minecraft::{get_minecraft_jvm_arguments, get_minecraft_version},
    wrap_ref_builder,
};

use super::{args, download_minecraft, download_version_info, download_version_manifest};

#[tracing::instrument]
pub async fn install_minecraft(
    instance: &Instance,
    loading_bar: Option<LoadingBarId>,
    force: bool,
) -> crate::Result<()> {
    log::info!(
        "Installing instance: \"{}\" (minecraft: \"{}\", modloader: \"{}\")",
        instance.name,
        instance.game_version,
        instance.loader_version.clone().unwrap_or_default()
    );

    let loading_bar = init_or_edit_loading(
        loading_bar,
        LoadingBarType::MinecraftDownload {
            instance_path: instance.id.clone(),
            instance_name: instance.name.clone(),
        },
        100.0,
        "Downloading Minecraft",
    )
    .await?;

    Instance::edit(&instance.id, |instance| {
        instance.install_stage = InstanceInstallStage::Installing;
        async { Ok(()) }
    })
    .await?;

    let result = async {
        let state = LauncherState::get().await?;

        let instance_path = Instance::get_full_path(&instance.id).await?;

        let version_manifest = download_version_manifest(&state, false).await?;

        let (version, minecraft_updated) = get_minecraft_version(instance, version_manifest)?;

        let mut loader_version = Instance::get_loader_version(
            &instance.game_version,
            instance.loader,
            instance.loader_version.as_deref(),
        )
        .await?;

        // If no loader version is selected, try to select the stable version!
        if instance.loader != ModLoader::Vanilla && loader_version.is_none() {
            loader_version = Instance::get_loader_version(
                &instance.game_version,
                instance.loader,
                Some("stable"),
            )
            .await?;

            let loader_version_id = loader_version.clone();

            Instance::edit(&instance.id, |instance| {
                instance.loader_version = loader_version_id.clone().map(|x| x.id.clone());
                async { Ok(()) }
            })
            .await?;
        }

        let version_jar = loader_version.as_ref().map_or(version.id.clone(), |it| {
            format!("{}-{}", version.id.clone(), it.id.clone())
        });

        let mut version_info = download_version_info(
            &state,
            &version,
            loader_version.as_ref(),
            Some(force),
            Some(&loading_bar),
        )
        .await?;

        let java = if let Some(java) = Instance::get_java(instance).await.transpose() {
            java
        } else {
            let compatible_java_version =
                crate::utils::minecraft::get_compatible_java_version(&version_info);

            let java = crate::api::java::get(compatible_java_version).await;

            match java {
                Ok(java) => Ok(java),
                Err(_) => crate::api::java::install(compatible_java_version).await,
            }
        }?;

        download_minecraft(
            &state,
            &version_info,
            &java.architecture,
            force,
            minecraft_updated,
            Some(&loading_bar),
        )
        .await?;

        mod_loader_post_install(
            instance,
            version_jar,
            &instance_path,
            &mut version_info,
            &java,
            Some(&loading_bar),
        )
        .await?;

        Instance::edit(&instance.id, |prof| {
            prof.install_stage = InstanceInstallStage::Installed;
            async { Ok(()) }
        })
        .await?;

        emit_loading(&loading_bar, 1.000_000_000_01, Some("Finished installing")).await?;

        Ok(())
    }
    .await;

    match result {
        Ok(_) => {
            log::info!(
                "Installed instance: \"{}\" (minecraft: \"{}\", modloader: \"{}\")",
                instance.name,
                instance.game_version,
                instance.loader_version.clone().unwrap_or_default()
            );
            Ok(())
        }
        Err(e) => {
            Instance::edit(&instance.id, |prof| {
                prof.install_stage = InstanceInstallStage::NotInstalled;
                async { Ok(()) }
            })
            .await?;

            Err(e)
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct InstanceLaunchArgs {
    pub env_args: Vec<(String, String)>,
    pub java_args: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct InstanceLaunchSettings {
    pub memory: MemorySettings,
    pub resolution: WindowSize,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct InstanceLaunchMetadata {
    pub post_exit_command: Option<String>,
    pub wrapper: Option<String>,
}

// TODO: reduce arguments count
#[tracing::instrument]
pub async fn launch_minecraft(
    instance: &Instance,
    launch_args: &InstanceLaunchArgs,
    launch_settings: &InstanceLaunchSettings,
    launch_metadata: &InstanceLaunchMetadata,
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

    let version_manifest = download_version_manifest(&state, false).await?;

    let (version, minecraft_updated) = get_minecraft_version(instance, version_manifest)?;

    let loader_version: Option<modded::LoaderVersion> = Instance::get_loader_version(
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
        download_version_info(&state, &version, loader_version.as_ref(), None, None).await?;

    let java = if let Some(java) = instance.get_java().await.transpose() {
        java
    } else {
        let compatible_java_version =
            crate::utils::minecraft::get_compatible_java_version(&version_info);

        crate::api::java::get(compatible_java_version).await
    }?;

    let client_path = state
        .locations
        .version_dir(&version_jar)
        .join(format!("{version_jar}.jar"));

    let args = version_info.arguments.clone().unwrap_or_default();

    let env_args_vec = launch_args.env_args.clone();

    let mut command = match &launch_metadata.wrapper {
        Some(hook) => {
            wrap_ref_builder!(it = tokio::process::Command::new(hook) => {it.arg(&java.path)})
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

    let jvm_arguments = get_minecraft_jvm_arguments(
        &state,
        &version_info,
        &natives_dir,
        &client_path,
        version_jar,
        &java,
        launch_settings.memory,
        &launch_args.java_args,
        &args,
        minecraft_updated,
    )?;

    let minecraft_arguments = args::get_minecraft_arguments(
        args.get(&minecraft::ArgumentType::Game)
            .map(|x| x.as_slice()),
        version_info.minecraft_arguments.as_deref(),
        credentials,
        &version.id,
        &version_info.asset_index.id,
        &instance_path,
        &state.locations.assets_dir(),
        &version.type_,
        launch_settings.resolution,
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
            launch_metadata.post_exit_command.clone(),
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
