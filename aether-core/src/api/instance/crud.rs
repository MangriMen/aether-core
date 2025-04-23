use std::path::{Path, PathBuf};

use crate::{
    core::LauncherState,
    features::{
        events::{emit::emit_instance, InstancePayloadType},
        instance::{
            self, instance::PackInfo, CreateInstanceDto, FsInstanceStorage, Instance,
            InstanceInstallStage,
        },
        minecraft::{install_minecraft, ModLoader},
        settings::{MemorySettings, WindowSize},
    },
    shared::read_json_async,
};

#[tracing::instrument]
pub async fn create(
    name: String,
    game_version: String,
    mod_loader: ModLoader,
    loader_version: Option<String>,
    icon_path: Option<String>,
    skip_install_instance: Option<bool>,
    pack_info: Option<PackInfo>,
) -> crate::Result<String> {
    let state = LauncherState::get().await?;

    let instance_storage = FsInstanceStorage::new(state.locations.clone());
    let metadata_storage = crate::api::metadata::get_storage().await?;

    let create_instance_dto = CreateInstanceDto {
        name,
        game_version,
        mod_loader,
        loader_version,
        icon_path,
        skip_install_instance,
        pack_info,
    };

    instance::create_instance(
        &instance_storage,
        &metadata_storage,
        &state.locations,
        &state.file_watcher,
        &create_instance_dto,
    )
    .await
}

#[tracing::instrument]
pub async fn install(id: &str, force: bool) -> crate::Result<()> {
    if let Ok(instance) = get(id).await {
        let result = install_minecraft(&instance, None, force).await;

        if result.is_err() && instance.install_stage != InstanceInstallStage::Installed {
            Instance::edit(id, |instance| {
                instance.install_stage = InstanceInstallStage::NotInstalled;
                async { Ok(()) }
            })
            .await?;
        }

        result?;
    } else {
        return Err(crate::ErrorKind::UnmanagedProfileError(id.to_string()).as_error());
    }

    Ok(())
}

#[tracing::instrument]
pub async fn update(id: &str) -> crate::Result<()> {
    if let Ok(instance) = get(id).await {
        if let Some(pack_info) = instance.pack_info {
            let state = LauncherState::get().await?;
            let plugin_manager = state.plugin_manager.read().await;

            if let Ok(plugin) = plugin_manager.get_plugin(&pack_info.pack_type) {
                if let Some(plugin) = plugin.get_plugin() {
                    plugin.lock().await.update(id).map_err(|_| {
                        crate::ErrorKind::InstanceUpdateError(format!(
                            "Failed to import instance from plugin {}",
                            pack_info.pack_type
                        ))
                        .as_error()
                    })?;
                } else {
                    return Err(crate::ErrorKind::InstanceUpdateError(format!(
                        "Can't get plugin \"{}\" to update instance. Check if it is installed and enabled",
                        &pack_info.pack_type
                    ))
                    .as_error());
                }

                return Ok(());
            } else {
                return Err(crate::ErrorKind::InstanceUpdateError(
                    "Unsupported pack type".to_owned(),
                )
                .as_error());
            };
        } else {
            return Err(
                crate::ErrorKind::InstanceUpdateError("There is not pack info".to_owned())
                    .as_error(),
            );
        };
    } else {
        return Err(crate::ErrorKind::UnmanagedProfileError(id.to_string()).as_error());
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

#[tracing::instrument]
pub async fn get_dir(id: &str) -> crate::Result<PathBuf> {
    let state = LauncherState::get().await?;
    Ok(state.locations.instance_dir(id))
}

#[tracing::instrument]
pub async fn get_file_path(id: &str) -> crate::Result<PathBuf> {
    Ok(get_dir(id).await?.join(".metadata").join("instance.json"))
}

#[tracing::instrument]
pub async fn get_by_path(path: &Path) -> crate::Result<Instance> {
    read_json_async(&path).await
}

#[tracing::instrument]
pub async fn get(id: &str) -> crate::Result<Instance> {
    get_by_path(&get_file_path(id).await?).await
}

#[tracing::instrument]
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
                let instance_file = entry.path().join(".metadata").join("instance.json");

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

#[tracing::instrument]
pub async fn remove(id: &str) -> crate::Result<()> {
    let instance = get(id).await?;
    instance.remove().await?;

    emit_instance(id, InstancePayloadType::Removed).await?;

    Ok(())
}
