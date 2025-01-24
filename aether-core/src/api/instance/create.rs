use std::fs::canonicalize;

use chrono::Utc;
use log::info;
use tokio::fs;

use crate::{
    api::instance::get_instance_path_without_duplicate,
    state::{
        Hooks, Instance, InstanceInstallStage, InstancePluginSettings, LauncherState,
        MemorySettings, ModLoader, WindowSize,
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
            Instance::remove_by_path(&full_path).await?;
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
