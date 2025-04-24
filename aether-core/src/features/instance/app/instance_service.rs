use std::sync::Arc;

use chrono::Utc;
use log::{error, info};

use crate::features::{
    events::{emit_instance, InstancePayloadType},
    instance::{
        create_instance_dir, watch_instance, FsWatcher, Instance, InstanceInstallStage,
        InstanceStorage, PackInfo,
    },
    minecraft::{self, install_minecraft, resolve_loader_version, ModLoader, ReadMetadataStorage},
    settings::{Hooks, LocationInfo},
};

use super::{EditInstance, NewInstance};

pub struct InstanceService<IS>
where
    IS: InstanceStorage,
{
    instance_storage: IS,
    location_info: Arc<LocationInfo>,
    fs_watcher: Arc<FsWatcher>,
}

impl<IS> InstanceService<IS>
where
    IS: InstanceStorage + Send + Sync,
{
    pub fn new(
        instance_storage: IS,
        location_info: Arc<LocationInfo>,
        fs_watcher: Arc<FsWatcher>,
    ) -> Self {
        Self {
            instance_storage,
            location_info,
            fs_watcher,
        }
    }

    pub async fn create<MS>(
        &self,
        metadata_storage: &MS,
        new_instance: &NewInstance,
    ) -> crate::Result<String>
    where
        MS: ReadMetadataStorage + ?Sized,
    {
        let NewInstance {
            name,
            game_version,
            mod_loader,
            loader_version,
            icon_path,
            skip_install_instance,
            pack_info,
        } = new_instance;

        let (instance_path, sanitized_name) =
            create_instance_dir(name, &self.location_info.instances_dir()).await?;

        info!(
            "Creating instance \"{}\" at path \"{:?}\"",
            &name, &instance_path
        );

        let loader_version = self
            .found_loader_version(metadata_storage, game_version, mod_loader, loader_version)
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

        let instance_id = self.setup_instance(&instance, skip_install_instance).await;

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
                if let Err(cleanup_err) = self.remove(&instance.id).await {
                    error!("Failed to cleanup instance: {}", cleanup_err);
                }
                Err(err)
            }
        }
    }

    pub async fn install(&self, id: &str, force: bool) -> crate::Result<()> {
        let mut instance = self.instance_storage.get(id).await?;

        if install_minecraft(&instance, None, force).await.is_err() {
            self.handle_failed_installation(&mut instance).await?;
        }

        Ok(())
    }

    pub async fn list(&self) -> crate::Result<Vec<Instance>> {
        Ok(self.instance_storage.list().await?)
    }

    pub async fn get(&self, id: &str) -> crate::Result<Instance> {
        Ok(self.instance_storage.get(id).await?)
    }

    pub async fn edit(&self, id: &str, edit_instance: &EditInstance) -> crate::Result<()> {
        validate_edit(edit_instance)?;
        let mut instance = self.instance_storage.get(id).await?;
        apply_edit_changes(&mut instance, edit_instance);
        self.upsert(&instance).await
    }

    pub async fn upsert(&self, instance: &Instance) -> crate::Result<()> {
        self.instance_storage.upsert(instance).await?;
        emit_instance(&instance.id, InstancePayloadType::Edited).await?;
        Ok(())
    }

    pub async fn remove(&self, id: &str) -> crate::Result<()> {
        self.instance_storage.remove(id).await?;
        emit_instance(id, InstancePayloadType::Removed).await
    }

    async fn found_loader_version<MS>(
        &self,
        metadata_storage: &MS,
        game_version: &str,
        mod_loader: &ModLoader,
        loader_version: &Option<String>,
    ) -> crate::Result<Option<daedalus::modded::LoaderVersion>>
    where
        MS: ReadMetadataStorage + ?Sized,
    {
        if !matches!(mod_loader, ModLoader::Vanilla) {
            let loader_version_manifest = metadata_storage
                .get_loader_version_manifest(mod_loader.as_meta_str())
                .await?
                .value;

            resolve_loader_version(
                game_version,
                mod_loader,
                loader_version.as_deref(),
                &loader_version_manifest,
            )
            .await
        } else {
            Ok(None)
        }
    }

    async fn setup_instance(
        &self,
        instance: &Instance,
        skip_install_instance: &Option<bool>,
    ) -> crate::Result<String> {
        self.upsert(instance).await?;

        watch_instance(&instance.id, &self.fs_watcher, &self.location_info).await;

        if !skip_install_instance.unwrap_or(false) {
            minecraft::install_minecraft(instance, None, false).await?;
        }

        Ok(instance.id.clone())
    }

    async fn handle_failed_installation(&self, instance: &mut Instance) -> crate::Result<()> {
        if instance.install_stage != InstanceInstallStage::Installed {
            instance.install_stage = InstanceInstallStage::NotInstalled;
            self.upsert(instance).await?;
        }
        Ok(())
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

fn apply_edit_changes(instance: &mut Instance, edit_instance: &EditInstance) {
    let EditInstance {
        name,
        java_path,
        extra_launch_args,
        custom_env_vars,
        memory,
        game_resolution,
    } = edit_instance;

    if let Some(name) = name {
        instance.name = name.clone();
    }

    if let Some(java_path) = java_path {
        instance.java_path = java_path.clone();
    }

    if let Some(args) = extra_launch_args {
        instance.extra_launch_args = args.clone();
    }

    if let Some(vars) = custom_env_vars {
        instance.custom_env_vars = vars.clone();
    }

    if let Some(mem) = memory {
        instance.memory = *mem;
    }

    if let Some(res) = game_resolution {
        instance.game_resolution = *res;
    }

    instance.modified = Utc::now();
}

fn validate_edit(edit: &EditInstance) -> crate::Result<()> {
    if let Some(name) = &edit.name {
        validate_name(name)?;
    }

    Ok(())
}

fn validate_name(name: &str) -> crate::Result<()> {
    if name.is_empty() {
        return Err(crate::ErrorKind::OtherError("Name cannot be empty".to_string()).into());
    }
    Ok(())
}
