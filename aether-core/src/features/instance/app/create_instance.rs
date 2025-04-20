use std::path::PathBuf;

use chrono::Utc;
use log::info;

use crate::{
    features::{
        events::{emit_instance, InstancePayloadType},
        instance::{
            create_instance_path_without_duplicate, instance::PackInfo, remove_instance,
            watch_instance, FsWatcher, Instance, InstanceInstallStage, InstanceStorage,
        },
        minecraft::{self, resolve_loader_version, ModLoader, ReadMetadataStorage},
        settings::{Hooks, LocationInfo},
    },
    shared::canonicalize,
};

pub async fn create_instance<IS, MS>(
    instance_storage: &IS,
    metadata_storage: &MS,
    location_info: &LocationInfo,
    fs_watcher: &FsWatcher,
    name: String,
    game_version: String,
    mod_loader: ModLoader,
    loader_version: Option<String>,
    icon_path: Option<String>,
    skip_install_instance: Option<bool>,
    pack_info: Option<PackInfo>,
) -> crate::Result<String>
where
    IS: InstanceStorage + ?Sized,
    MS: ReadMetadataStorage + ?Sized,
{
    let (full_path, sanitized_name) =
        create_instance_path_without_duplicate(&location_info.instances_dir(), &name);

    let canonicalized_path = &canonicalize(&full_path);

    info!(
        "Creating instance \"{}\" at path \"{:?}\"",
        &sanitized_name, &canonicalized_path
    );

    tokio::fs::create_dir_all(&full_path).await?;

    let loader_version_manifest = metadata_storage
        .get_loader_version_manifest(mod_loader.as_meta_str())
        .await?
        .value;

    let loader = resolve_loader_version(
        &game_version,
        mod_loader,
        loader_version.as_deref(),
        &loader_version_manifest,
    )
    .await?;

    let instance = build_instance(
        name,
        sanitized_name.clone(),
        full_path.clone(),
        game_version,
        mod_loader,
        loader,
        icon_path,
        pack_info,
    );

    let result = async {
        instance_storage.upsert(&instance).await?;

        watch_instance(&instance.id, fs_watcher, location_info).await;
        emit_instance(&instance.id, InstancePayloadType::Created).await?;

        if !skip_install_instance.unwrap_or(false) {
            minecraft::install_minecraft(&instance, None, false).await?;
        }

        Ok(instance.id.clone())
    }
    .await;

    match result {
        Ok(path) => {
            info!(
                "Instance \"{}\" created successfully at path \"{:?}\"",
                &instance.name, canonicalized_path
            );
            Ok(path)
        }
        Err(err) => {
            info!(
                "Failed to create instance \"{}\". Instance removed",
                &instance.name
            );
            remove_instance(instance_storage, location_info, &instance.id).await?;
            Err(err)
        }
    }
}

fn build_instance(
    name: String,
    sanitized_name: String,
    full_path: PathBuf,
    game_version: String,
    mod_loader: ModLoader,
    loader: Option<daedalus::modded::LoaderVersion>,
    icon_path: Option<String>,
    pack_info: Option<PackInfo>,
) -> Instance {
    Instance {
        id: sanitized_name,
        path: full_path,
        name,
        icon_path,
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
        pack_info,
    }
}
