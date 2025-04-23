use std::path::{Path, PathBuf};

use chrono::Utc;
use log::{error, info};

use crate::features::{
    events::{emit_instance, InstancePayloadType},
    instance::{
        create_instance_path_without_duplicate, instance::PackInfo, remove_instance,
        watch_instance, FsWatcher, Instance, InstanceInstallStage, InstanceStorage,
    },
    minecraft::{self, resolve_loader_version, ModLoader, ReadMetadataStorage},
    settings::{Hooks, LocationInfo},
};

pub struct CreateInstanceDto {
    pub name: String,
    pub game_version: String,
    pub mod_loader: ModLoader,
    pub loader_version: Option<String>,
    pub icon_path: Option<String>,
    pub skip_install_instance: Option<bool>,
    pub pack_info: Option<PackInfo>,
}

pub async fn create_instance<IS, MS>(
    instance_storage: &IS,
    metadata_storage: &MS,
    location_info: &LocationInfo,
    fs_watcher: &FsWatcher,
    create_instance_dto: &CreateInstanceDto,
) -> crate::Result<String>
where
    IS: InstanceStorage + ?Sized,
    MS: ReadMetadataStorage + ?Sized,
{
    let CreateInstanceDto {
        name,
        game_version,
        mod_loader,
        loader_version,
        icon_path,
        skip_install_instance,
        pack_info,
    } = create_instance_dto;

    let (instance_path, sanitized_name) =
        create_instance_dir(name, &location_info.instances_dir()).await?;

    info!(
        "Creating instance \"{}\" at path \"{:?}\"",
        &name, &instance_path
    );

    let loader_version_manifest = metadata_storage
        .get_loader_version_manifest(mod_loader.as_meta_str())
        .await?
        .value;

    let loader_version = resolve_loader_version(
        game_version,
        mod_loader,
        loader_version.as_deref(),
        &loader_version_manifest,
    )
    .await?;

    let instance = build_instance(
        name,
        &sanitized_name,
        game_version,
        *mod_loader,
        loader_version.as_ref(),
        icon_path,
        pack_info,
    );

    let instance_id = setup_instance(
        instance_storage,
        location_info,
        fs_watcher,
        &instance,
        skip_install_instance,
    )
    .await;

    match instance_id {
        Ok(instance_id) => {
            info!(
                "Instance \"{}\" created successfully at path \"{:?}\"",
                &instance.name, &instance_path
            );
            Ok(instance_id)
        }
        Err(err) => {
            info!(
                "Failed to create instance \"{}\". Rolling back",
                &instance.name
            );
            if let Err(cleanup_err) =
                remove_instance(instance_storage, location_info, &instance.id).await
            {
                error!("Failed to cleanup instance: {}", cleanup_err);
            }
            Err(err)
        }
    }
}

async fn create_instance_dir(name: &str, base_dir: &Path) -> crate::Result<(PathBuf, String)> {
    let (instance_path, sanitized_name) = create_instance_path_without_duplicate(name, base_dir);
    tokio::fs::create_dir_all(&instance_path).await?;
    Ok((instance_path, sanitized_name))
}

fn build_instance(
    name: &str,
    sanitized_name: &str,
    game_version: &str,
    mod_loader: ModLoader,
    loader_version: Option<&daedalus::modded::LoaderVersion>,
    icon_path: &Option<String>,
    pack_info: &Option<PackInfo>,
) -> Instance {
    Instance {
        id: sanitized_name.to_owned(),
        name: name.to_owned(),
        icon_path: icon_path.as_ref().map(ToOwned::to_owned),
        install_stage: InstanceInstallStage::NotInstalled,
        game_version: game_version.to_owned(),
        loader: mod_loader,
        loader_version: loader_version.map(|v| v.id.clone()),
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
        pack_info: pack_info.clone(),
    }
}

async fn setup_instance<IS>(
    instance_storage: &IS,
    location_info: &LocationInfo,
    fs_watcher: &FsWatcher,
    instance: &Instance,
    skip_install_instance: &Option<bool>,
) -> crate::Result<String>
where
    IS: InstanceStorage + ?Sized,
{
    instance_storage.upsert(instance).await?;

    watch_instance(&instance.id, fs_watcher, location_info).await;
    emit_instance(&instance.id, InstancePayloadType::Created).await?;

    if !skip_install_instance.unwrap_or(false) {
        minecraft::install_minecraft(instance, None, false).await?;
    }

    Ok(instance.id.clone())
}
