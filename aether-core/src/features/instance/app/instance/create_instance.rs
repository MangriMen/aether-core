use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use chrono::Utc;
use extism::{FromBytes, ToBytes};
use extism_convert::{encoding, Json};
use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::{
    features::{
        events::{EventEmitter, EventEmitterExt, ProgressService},
        instance::{
            InstallInstanceUseCase, Instance, InstanceError, InstanceInstallStage, InstanceStorage,
            InstanceWatcherService, PackInfo,
        },
        java::{JavaInstallationService, JavaStorage},
        minecraft::{
            LoaderVersionPreference, LoaderVersionResolver, MinecraftDownloader, ModLoader,
            ReadMetadataStorage,
        },
        settings::{Hooks, LocationInfo},
    },
    libs::request_client::RequestClient,
    shared::create_dir_all,
};

#[derive(Debug, Serialize, Deserialize, FromBytes, ToBytes)]
#[encoding(Json)]
#[serde(rename_all = "camelCase")]
pub struct NewInstance {
    pub name: String,
    pub game_version: String,
    pub mod_loader: ModLoader,
    pub loader_version: Option<LoaderVersionPreference>,
    pub icon_path: Option<String>,
    pub skip_install_instance: Option<bool>,
    pub pack_info: Option<PackInfo>,
}

pub struct CreateInstanceUseCase<
    IS: InstanceStorage,
    MS: ReadMetadataStorage,
    E: EventEmitter,
    MD: MinecraftDownloader,
    PS: ProgressService,
    IWS: InstanceWatcherService,
    JIS: JavaInstallationService,
    JS: JavaStorage,
    RC: RequestClient,
> {
    instance_storage: Arc<IS>,
    loader_version_resolver: Arc<LoaderVersionResolver<MS>>,
    install_instance_use_case: Arc<InstallInstanceUseCase<IS, MS, MD, PS, JIS, JS, RC>>,
    location_info: Arc<LocationInfo>,
    event_emitter: Arc<E>,
    instance_watcher_service: Arc<IWS>,
}

impl<
        IS: InstanceStorage,
        MS: ReadMetadataStorage,
        E: EventEmitter,
        MD: MinecraftDownloader,
        PS: ProgressService,
        IWS: InstanceWatcherService,
        JIS: JavaInstallationService,
        JS: JavaStorage,
        RC: RequestClient,
    > CreateInstanceUseCase<IS, MS, E, MD, PS, IWS, JIS, JS, RC>
{
    pub fn new(
        instance_storage: Arc<IS>,
        loader_version_resolver: Arc<LoaderVersionResolver<MS>>,
        install_instance_use_case: Arc<InstallInstanceUseCase<IS, MS, MD, PS, JIS, JS, RC>>,
        location_info: Arc<LocationInfo>,
        event_emitter: Arc<E>,
        instance_watcher_service: Arc<IWS>,
    ) -> Self {
        Self {
            instance_storage,
            loader_version_resolver,
            install_instance_use_case,
            location_info,
            event_emitter,
            instance_watcher_service,
        }
    }
    async fn setup_instance(
        &self,
        instance: &Instance,
        skip_install_instance: Option<bool>,
    ) -> Result<String, InstanceError> {
        self.instance_storage.upsert(instance).await?;

        self.instance_watcher_service
            .watch_instance(&instance.id)
            .await?;

        if !skip_install_instance.unwrap_or(false) {
            self.install_instance_use_case
                .execute(instance.id.clone(), false)
                .await?;
        }

        Ok(instance.id.clone())
    }

    pub async fn execute(&self, new_instance: NewInstance) -> Result<String, InstanceError> {
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
            .resolve(&game_version, &mod_loader, loader_version.as_ref())
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
                self.event_emitter
                    .emit_warning(format!("Error creating instance {}", err))
                    .await
                    .unwrap_or_else(|e| {
                        log::error!("Error emitting warning: {}", e);
                    });
                if let Err(cleanup_err) = self.instance_storage.remove(&instance.id).await {
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
        loader_version: loader_version.map(|v| LoaderVersionPreference::Exact(v.id.clone())),
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
) -> Result<(PathBuf, String), InstanceError> {
    let (instance_path, sanitized_name) = create_unique_instance_path(name, base_dir);
    create_dir_all(&instance_path).await?;
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
