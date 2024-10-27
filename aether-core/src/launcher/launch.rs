use std::path::PathBuf;

use daedalus::{minecraft, modded};

use crate::{
    api,
    event::{emit_loading, init_or_edit_loading, LoadingBarId, LoadingBarType},
    launcher::process_forge_processors,
    state::{
        self, Instance, InstanceInstallStage, LauncherState, MinecraftProcessMetadata, ModLoader,
    },
    utils::minecraft::{get_minecraft_jvm_arguments, get_minecraft_version},
    wrap_ref_builder,
};

use super::{args, download_minecraft, download_version_info, download_version_manifest};

#[tracing::instrument]
pub async fn install_minecraft(
    instance: &Instance,
    loading_bar: Option<LoadingBarId>,
    repairing: bool,
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
            instance_path: instance.name_id.clone(),
            instance_name: instance.name.clone(),
        },
        100.0,
        "Downloading Minecraft",
    )
    .await?;

    Instance::edit(&instance.name_id, |instance| {
        instance.install_stage = InstanceInstallStage::Installing;
        async { Ok(()) }
    })
    .await?;

    let state = LauncherState::get().await?;

    let instance_path = Instance::get_full_path(&instance.name_id).await?;

    let version_manifest = download_version_manifest(&state, false).await?;

    let (version, minecraft_updated) = get_minecraft_version(instance, version_manifest)?;

    let mut loader_version = Instance::get_loader_version_from_instance(
        &instance.game_version,
        instance.loader,
        instance.loader_version.as_deref(),
    )
    .await?;

    // If no loader version is selected, try to select the stable version!
    if instance.loader != ModLoader::Vanilla && loader_version.is_none() {
        loader_version = Instance::get_loader_version_from_instance(
            &instance.game_version,
            instance.loader,
            Some("stable"),
        )
        .await?;

        let loader_version_id = loader_version.clone();

        Instance::edit(&instance.name_id, |instance| {
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
        Some(repairing),
        Some(&loading_bar),
    )
    .await?;

    let compatible_java_version = version_info
        .java_version
        .as_ref()
        .map(|it| it.major_version)
        .unwrap_or(8);

    let (java_version, set_java) = if let Some(java_version) =
        Instance::get_java_version_from_instance(instance, &version_info).await?
    {
        (PathBuf::from(java_version.path), false)
    } else {
        let path = crate::jre::auto_install_java(compatible_java_version).await?;
        (path, true)
    };

    let java_version = crate::api::jre::check_jre(java_version.clone())
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::LauncherError(format!(
                "Java path invalid or non-functional: {:?}",
                java_version
            ))
        })?;

    if set_java {
        java_version.update(&state).await?;
    }

    download_minecraft(
        &state,
        &version_info,
        &java_version.architecture,
        repairing,
        minecraft_updated,
        Some(&loading_bar),
    )
    .await?;

    process_forge_processors(
        instance,
        version_jar,
        &instance_path,
        &mut version_info,
        &java_version,
        Some(&loading_bar),
    )
    .await?;

    Instance::edit(&instance.name_id, |prof| {
        prof.install_stage = InstanceInstallStage::Installed;
        async { Ok(()) }
    })
    .await?;

    // TODO: must be 1.0, but now it is 0.99997 at the end, need fix
    emit_loading(&loading_bar, 1.0, Some("Finished installing")).await?;

    Ok(())
}

#[tracing::instrument]
pub async fn launch_minecraft(
    instance: &Instance,
    env_args: &[(String, String)],
    java_args: &[String],
    memory: &state::MemorySettings,
    resolution: &state::WindowSize,
    credentials: &state::Credentials,
    post_exit_command: Option<String>,
    wrapper: &Option<String>,
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

    let instance_path = Instance::get_full_path(&instance.name_id).await?;

    let version_manifest = download_version_manifest(&state, false).await?;

    let (version, minecraft_updated) = get_minecraft_version(instance, version_manifest)?;

    let loader_version: Option<modded::LoaderVersion> = Instance::get_loader_version_from_instance(
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

    let java_version = instance
        .get_java_version_from_instance(&version_info)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::LauncherError("Missing correct java installation".to_string())
        })?;

    let java_version = crate::api::jre::check_jre(java_version.path.clone().into())
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::LauncherError(format!(
                "Java path invalid or non-functional: {}",
                java_version.path
            ))
        })?;

    let client_path = state
        .locations
        .version_dir(&version_jar)
        .join(format!("{version_jar}.jar"));

    let args = version_info.arguments.clone().unwrap_or_default();

    let env_args_vec = Vec::from(env_args);

    let mut command = match wrapper {
        Some(hook) => {
            wrap_ref_builder!(it = tokio::process::Command::new(hook) => {it.arg(&java_version.path)})
        }
        None => tokio::process::Command::new(&java_version.path),
    };

    // Check if profile has a running profile, and reject running the command if it does
    // Done late so a quick double call doesn't launch two instances
    let existing_processes = api::process::get_by_instance_name_id(&instance.name_id).await?;
    if let Some(process) = existing_processes.first() {
        return Err(crate::ErrorKind::LauncherError(format!(
            "Profile {} is already running at path: {}",
            instance.name_id, process.uuid
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
        &java_version,
        *memory,
        java_args,
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
        *resolution,
        &java_version.architecture,
    )?
    .into_iter()
    .collect::<Vec<_>>();

    // log::error!(instance)

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

    // minimize launcher

    // run process

    state
        .process_manager
        .insert_new_process(&instance.name_id, command, post_exit_command)
        .await
}
