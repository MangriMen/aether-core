use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use chrono::Utc;
use log::{error, info};

use crate::{
    features::{
        instance::{
            watch_instance, FsWatcher, Instance, InstanceInstallStage, InstanceManager, PackInfo,
        },
        minecraft::{self, LoaderVersionResolver, ModLoader, ReadMetadataStorage},
        settings::{Hooks, LocationInfo},
    },
    shared::domain::AsyncUseCaseWithInputAndError,
};

use super::NewInstance;

pub struct CreateInstanceUseCase<IM: InstanceManager, MS: ReadMetadataStorage> {
    instance_manager: Arc<IM>,
    loader_version_resolver: Arc<LoaderVersionResolver<MS>>,
    location_info: Arc<LocationInfo>,
    fs_watcher: Arc<FsWatcher>,
}

impl<IM: InstanceManager, MS: ReadMetadataStorage> CreateInstanceUseCase<IM, MS> {
    pub fn new(
        instance_manager: Arc<IM>,
        loader_version_resolver: Arc<LoaderVersionResolver<MS>>,
        location_info: Arc<LocationInfo>,
        fs_watcher: Arc<FsWatcher>,
    ) -> Self {
        Self {
            instance_manager,
            loader_version_resolver,
            location_info,
            fs_watcher,
        }
    }
    async fn setup_instance(
        &self,
        instance: &Instance,
        skip_install_instance: Option<bool>,
    ) -> crate::Result<String> {
        self.instance_manager.upsert(instance).await?;

        watch_instance(&instance.id, &self.fs_watcher, &self.location_info).await;

        if !skip_install_instance.unwrap_or(false) {
            minecraft::install_minecraft(
                &*self.instance_manager,
                &self.loader_version_resolver,
                instance,
                None,
                false,
            )
            .await?;
        }

        Ok(instance.id.clone())
    }
}

#[async_trait]
impl<IM, MS> AsyncUseCaseWithInputAndError for CreateInstanceUseCase<IM, MS>
where
    IM: InstanceManager + Send + Sync,
    MS: ReadMetadataStorage + Send + Sync,
{
    type Input = NewInstance;
    type Output = String;
    type Error = crate::Error;

    async fn execute(&self, new_instance: Self::Input) -> Result<Self::Output, Self::Error> {
        let NewInstance {
            name,
            game_version,
            mod_loader,
            loader_version,
            icon_path,
            skip_install_instance,
            pack_info,
        } = new_instance;

        let (instance_dir, sanitized_name) =
            create_unique_instance_dir(&name, &self.location_info.instances_dir()).await?;

        info!(
            "Creating instance \"{}\" at path \"{:?}\"",
            &name, &instance_dir
        );

        let loader_version = self
            .loader_version_resolver
            .resolve(&game_version, &mod_loader, &loader_version)
            .await?;

        let instance = build_instance(
            &name,
            &sanitized_name,
            &game_version,
            mod_loader,
            loader_version.as_ref(),
            &icon_path,
            &pack_info,
        );

        let instance_id = self.setup_instance(&instance, skip_install_instance).await;

        match instance_id {
            Ok(instance_id) => {
                info!(
                    "Instance \"{}\" created successfully at path \"{:?}\"",
                    &instance.name, &instance_dir
                );
                Ok(instance_id)
            }
            Err(err) => {
                info!(
                    "Failed to create instance \"{}\". Rolling back",
                    &instance.name
                );
                if let Err(cleanup_err) = self.instance_manager.remove(&instance.id).await {
                    error!("Failed to cleanup instance: {}", cleanup_err);
                }
                Err(err)
            }
        }
    }
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

async fn create_unique_instance_dir(
    name: &str,
    base_dir: &Path,
) -> crate::Result<(PathBuf, String)> {
    let (instance_path, sanitized_name) = create_unique_instance_path(name, base_dir);
    tokio::fs::create_dir_all(&instance_path).await?;
    Ok((instance_path, sanitized_name))
}

fn create_unique_instance_path(name: &str, base_dir: &Path) -> (PathBuf, String) {
    let base_sanitized_name = sanitize_instance_name(name);

    let mut sanitized_name = base_sanitized_name.clone();
    let mut full_path = base_dir.join(&sanitized_name);

    let mut counter = 1;
    while full_path.exists() {
        sanitized_name = format!("{}-{}", base_sanitized_name, counter);
        full_path = base_dir.join(&sanitized_name);
        counter += 1;
    }

    (full_path, sanitized_name)
}

pub fn sanitize_instance_name(name: &str) -> String {
    name.replace(
        ['/', '\\', '?', '*', ':', '\'', '\"', '|', '<', '>', '!'],
        "_",
    )
}
