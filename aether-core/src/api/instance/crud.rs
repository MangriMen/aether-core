use std::path::{Path, PathBuf};

use crate::{
    event::emit::emit_instance,
    state::{Instance, LauncherState},
    utils::io::read_json_async,
};

use std::fs::canonicalize;

use chrono::Utc;
use log::info;
use tokio::fs;

use crate::{
    api::instance::get_instance_path_without_duplicate,
    state::{
        Hooks, InstanceInstallStage, InstancePluginSettings, MemorySettings, ModLoader, WindowSize,
    },
};

#[tracing::instrument]
pub async fn create(
    name: String,
    game_version: String,
    mod_loader: ModLoader,
    loader_version: Option<String>,
    icon_path: Option<String>,
    skip_install_instance: Option<bool>,
    plugin: Option<InstancePluginSettings>,
) -> crate::Result<String> {
    let state = LauncherState::get().await?;

    let (full_path, sanitized_name) = get_instance_path_without_duplicate(&state, &name);

    fs::create_dir_all(&full_path).await?;

    info!(
        "Creating instance \"{}\" at path \"{}\"",
        &sanitized_name,
        &canonicalize(&full_path)?.display()
    );

    let loader = if mod_loader != ModLoader::Vanilla {
        Instance::get_loader_version(&game_version, mod_loader, loader_version.as_deref()).await?
    } else {
        None
    };

    let instance = Instance {
        id: sanitized_name.clone(),
        path: full_path.clone(),

        name: name.clone(),
        icon_path: None,

        install_stage: InstanceInstallStage::NotInstalled,

        game_version,
        loader: mod_loader,
        loader_version: loader.map(|it| it.id),

        java_path: None,
        extra_launch_args: None,
        custom_env_vars: None,

        memory: None,
        force_fullscreen: None,
        game_resolution: None,

        created: Utc::now(),
        modified: Utc::now(),
        last_played: None,

        time_played: 0,
        recent_time_played: 0,

        hooks: Hooks::default(),

        plugin,
    };

    let result = async {
        instance.save().await?;

        crate::state::watch_instance(&instance.id, &state.file_watcher, &state.locations).await;

        emit_instance(&instance.id, crate::event::InstancePayloadType::Created).await?;

        if !skip_install_instance.unwrap_or(false) {
            crate::launcher::install_minecraft(&instance, None, false).await?;
        }

        Ok(instance.id.clone())
    }
    .await;

    match result {
        Ok(path) => {
            info!(
                "Instance \"{}\" created successfully at path \"{}\"",
                &sanitized_name,
                &canonicalize(&full_path)?.display()
            );
            Ok(path)
        }
        Err(err) => {
            info!("Failed to create instance \"{}\". Instance removed", &name);
            instance.remove().await?;
            Err(err)
        }
    }
}

#[tracing::instrument]
pub async fn edit(
    id: &str,
    name: &Option<String>,
    java_path: &Option<String>,
    extra_launch_args: &Option<Vec<String>>,
    custom_env_vars: &Option<Vec<(String, String)>>,
    memory: &Option<MemorySettings>,
    game_resolution: &Option<WindowSize>,
) -> crate::Result<()> {
    Instance::edit(id, |instance| {
        if let Some(name) = name.clone() {
            instance.name = name;
        }

        if let Some(java_path) = java_path.clone() {
            instance.java_path = Some(java_path);
        }

        instance.extra_launch_args = extra_launch_args.clone();
        instance.custom_env_vars = custom_env_vars.clone();
        instance.memory = *memory;
        instance.game_resolution = *game_resolution;

        async { Ok(()) }
    })
    .await
}

pub async fn get_dir(id: &str) -> crate::Result<PathBuf> {
    let state = LauncherState::get().await?;
    Ok(state.locations.instance_dir(id))
}

pub async fn get_file_path(id: &str) -> crate::Result<PathBuf> {
    Ok(get_dir(id).await?.join("instance.json"))
}

pub async fn get_by_path(path: &Path) -> crate::Result<Instance> {
    read_json_async(&path).await
}

pub async fn get(id: &str) -> crate::Result<Instance> {
    get_by_path(&get_file_path(id).await?).await
}

pub async fn get_all() -> crate::Result<(Vec<Instance>, Vec<crate::Error>)> {
    let state = LauncherState::get().await?;

    let instances_dir = state.locations.instances_dir();

    if !instances_dir.exists() {
        return Ok((Vec::new(), Vec::new()));
    }

    let mut instances = Vec::new();
    let mut instances_errors: Vec<crate::Error> = Vec::new();

    for entry in instances_dir.read_dir()? {
        match entry {
            Ok(entry) => {
                let instance_file = entry.path().join("instance.json");

                let instance = get_by_path(&instance_file).await;

                match instance {
                    Ok(instance) => {
                        instances.push(instance);
                    }
                    Err(err) => instances_errors.push(err),
                }
            }
            Err(err) => instances_errors.push(err.into()),
        }
    }

    Ok((instances, instances_errors))
}

pub async fn remove(id: &str) -> crate::Result<()> {
    let instance = get(id).await?;
    instance.remove().await?;

    emit_instance(id, crate::event::InstancePayloadType::Removed).await?;

    Ok(())
}
