use std::{fs::canonicalize, path::PathBuf};

use chrono::Utc;
use log::info;
use tokio::fs;

use crate::state::{Instance, InstanceInstallStage, LauncherState, ModLoader};

use super::sanitize_instance_name;

#[tracing::instrument]
pub async fn instance_create(
    name: String,
    game_version: String,
    mod_loader: ModLoader,
    loader_version: Option<String>,
    icon_path: Option<String>,
    linked_data: Option<String>,
    skip_install_profile: Option<bool>,
) -> anyhow::Result<String> {
    let state = LauncherState::get().await?;

    let (full_path, sanitized_name) = get_instance_path_without_duplicate(&state, &name);

    fs::create_dir_all(&full_path).await?;

    info!(
        "Created profile \"{}\" at path \"{}\"",
        &sanitized_name,
        &canonicalize(&full_path)?.display()
    );

    let loader = if mod_loader != ModLoader::Vanilla {
        Instance::get_loader_version_from_instance(
            &game_version,
            mod_loader,
            loader_version.as_deref(),
        )
        .await?
    } else {
        None
    };

    let instance = Instance {
        name_id: sanitized_name.clone(),
        install_stage: InstanceInstallStage::NotInstalled,

        path: full_path.clone(),

        name: name.clone(),
        icon_path: None,

        game_version,
        loader: mod_loader,
        loader_version: loader.map(|it| it.id),
        created: Utc::now(),
        modified: Utc::now(),
        last_played: None,
        java_path: None,
        extra_launch_args: None,
        custom_env_vars: None,
        memory: None,
        force_fullscreen: None,
        game_resolution: None,
        time_played: 0,
    };

    let result = async {
        instance.save().await?;

        if !skip_install_profile.unwrap_or(false) {
            crate::launcher::install_minecraft(&instance, None, false).await?;
        }

        Ok(instance.name_id.clone())
    }
    .await;

    match result {
        Ok(path) => Ok(path),
        Err(err) => {
            let _ = instance.remove().await;

            Err(err)
        }
    }
}

pub fn get_instance_path_without_duplicate(state: &LauncherState, name: &str) -> (PathBuf, String) {
    let mut sanitized_name = sanitize_instance_name(name);
    let mut full_path = state.locations.instances_dir().join(&sanitized_name);

    if full_path.exists() {
        let mut new_sanitized_name;
        let mut new_full_path;
        let mut which = 1;

        loop {
            new_sanitized_name = format!("{}-{}", sanitized_name, which);
            new_full_path = state.locations.instances_dir().join(&new_sanitized_name);

            if !new_full_path.exists() {
                break;
            }

            which += 1;
        }

        sanitized_name = new_sanitized_name;
        full_path = new_full_path;
    }

    (full_path, sanitized_name)
}
