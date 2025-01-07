use std::path::PathBuf;

use async_trait::async_trait;
use phf::phf_map;
use reqwest::Method;
use tokio::{
    fs::{create_dir_all, hard_link, remove_dir_all},
    process::Command,
};
use url::Url;

use crate::{
    api::{
        self,
        instance::{get_dir, instance_create},
    },
    event::{emit_loading, LoadingBarId},
    state::{LauncherState, ModLoader},
    utils::{
        fetch::{fetch_advanced, fetch_json},
        io::{read_toml_async, write_async, write_toml_async},
    },
};

use super::InstancePlugin;

pub struct PackwizPlugin {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub enum PackwizPluginDataType {
    Import,
    Update,
}

impl PackwizPluginDataType {
    pub fn as_str(&self) -> &'static str {
        match *self {
            Self::Import => "import",
            Self::Update => "update",
        }
    }

    pub fn from_string(val: &str) -> Self {
        match val {
            "import" => Self::Import,
            "update" => Self::Update,
            _ => Self::Import,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum PackwizPluginData {
    Import { url: String },
    Update { instance_id: String },
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Eq, Hash, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct PackwizPack {
    pub name: String,
    pub author: String,
    pub version: String,
    pub pack_format: Option<String>,
    pub description: Option<String>,

    pub index: PackIndex,
    pub versions: PackVersions,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Eq, Hash, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct PackIndex {
    file: String,
    hash_format: String,
    hash: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Eq, Hash, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct PackVersions {
    minecraft: String,
    fabric: Option<String>,
    forge: Option<String>,
    liteloader: Option<String>,
    quilt: Option<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum PackwizPluginError {
    #[error("Unsupported mod loader: {0}")]
    UnsupportedModLoader(String),
}
impl From<PackwizPluginError> for crate::ErrorKind {
    fn from(err: PackwizPluginError) -> Self {
        Self::PluginError(err.to_string())
    }
}
#[derive(serde::Serialize, serde::Deserialize)]
pub struct PackwizSettings {
    path: String,
}

impl PackwizPlugin {
    const PACKWIZ_INSTALLER_NAME: &str = "packwiz-installer.jar";
    const PACKWIZ_INSTALLER_BOOTSTRAP_NAME: &str = "packwiz-installer-bootstrap.jar";

    pub const REDISTRIBUTABLE_FILES: phf::Map<&'static str, &str> = phf_map! {
        "packwiz-installer" => "packwiz-installer.jar",
        "packwiz-installer-bootstrap" => "packwiz-installer-bootstrap.jar"
    };

    pub const REDISTRIBUTABLE_LINKS: phf::Map<&'static str, &str> = phf_map! {
        "packwiz-installer" => "https://github.com/packwiz/packwiz-installer/releases/latest/download/packwiz-installer.jar",
        "packwiz-installer-bootstrap" => "https://github.com/packwiz/packwiz-installer-bootstrap/releases/latest/download/packwiz-installer-bootstrap.jar"
    };

    fn get_instance_settings(&self, folder: &PathBuf) -> PathBuf {
        folder.join("packwiz.toml")
    }

    async fn get_pack_from_url_or_path(
        &self,
        state: &LauncherState,
        path_or_url: &str,
    ) -> crate::Result<PackwizPack> {
        log::info!("Get pack from {}", path_or_url);

        let path = PathBuf::from(path_or_url);

        if path.exists() && path.metadata().is_ok() {
            read_toml_async::<PackwizPack>(&PathBuf::from(path_or_url)).await
        } else {
            let url = Url::parse(path_or_url)?;

            fetch_json::<PackwizPack>(
                Method::GET,
                url.as_str(),
                None,
                None,
                None,
                &state.fetch_semaphore,
            )
            .await
        }
    }

    async fn load_settings_from_instance_folder(
        &self,
        folder: &PathBuf,
    ) -> crate::Result<PackwizSettings> {
        Ok(read_toml_async::<PackwizSettings>(&self.get_instance_settings(&folder)).await?)
    }

    async fn save_settings(
        &self,
        folder: &PathBuf,
        settings: &PackwizSettings,
    ) -> crate::Result<()> {
        Ok(write_toml_async(&self.get_instance_settings(&folder), &settings).await?)
    }

    fn extract_mod_loader(
        version: &PackVersions,
    ) -> Result<(ModLoader, Option<String>), PackwizPluginError> {
        match (
            version.fabric.as_ref(),
            version.forge.as_ref(),
            version.liteloader.as_ref(),
            version.quilt.as_ref(),
        ) {
            (Some(fabric), _, _, _) => Ok((ModLoader::Fabric, Some(fabric.clone()))),
            (_, Some(forge), _, _) => Ok((ModLoader::Forge, Some(forge.clone()))),
            (_, _, Some(liteloader), _) => Err(PackwizPluginError::UnsupportedModLoader(format!(
                "liteloader: {}",
                liteloader
            ))),
            (_, _, _, Some(quilt)) => Ok((ModLoader::Quilt, Some(quilt.clone()))),
            (_, _, _, _) => Ok((ModLoader::Vanilla, None)),
        }
    }

    async fn get_command_to_update_pack(
        &self,
        settings: &PackwizSettings,
        instance_folder: &PathBuf,
    ) -> crate::Result<Command> {
        const JAVA_VERSION: u32 = 8;
        let java = api::jre::get_or_download_java(JAVA_VERSION).await?;

        let plugin_dir = <dyn InstancePlugin>::get_plugin_dir(self).await?;

        let packwiz_installer_paths = (
            plugin_dir.join(Self::PACKWIZ_INSTALLER_NAME),
            plugin_dir.join(Self::PACKWIZ_INSTALLER_BOOTSTRAP_NAME),
        );

        let packwiz_installer_instance_path = instance_folder.join(Self::PACKWIZ_INSTALLER_NAME);

        if !packwiz_installer_instance_path.exists() {
            // TODO: need admin rights
            // symlink_file(
            //     &packwiz_installer_paths.1,
            //     &packwiz_installer_paths.0,
            // )
            // .await?;
            hard_link(&packwiz_installer_paths.0, &packwiz_installer_instance_path).await?;
        }

        let mut cmd = Command::new(java.path.to_string());
        cmd.current_dir(instance_folder)
            .arg("-jar")
            .arg(&packwiz_installer_paths.1)
            .arg("--bootstrap-no-update")
            .arg(settings.path.clone());
        Ok(cmd)
    }

    async fn download_file(
        &self,
        state: &LauncherState,
        url: &str,
        path: &PathBuf,
        loading_bar: Option<(&LoadingBarId, f64)>,
    ) -> crate::Result<()> {
        if path.exists() && path.metadata().is_ok() {
            if let Some(bar) = &loading_bar {
                emit_loading(bar.0, bar.1, None).await?;
            }
            return Ok(());
        }

        let body = fetch_advanced(
            Method::GET,
            url,
            None,
            None,
            None,
            &state.fetch_semaphore,
            loading_bar,
        )
        .await?;

        write_async(path, body).await?;

        Ok(())
    }

    async fn download_redistributable(&self, state: &LauncherState) -> crate::Result<()> {
        let plugin_dir = <dyn InstancePlugin>::get_plugin_dir(self).await?;

        let redistributable_paths: std::collections::HashMap<&str, PathBuf> =
            Self::REDISTRIBUTABLE_FILES
                .into_iter()
                .map(|(key, file_name)| (*key, plugin_dir.join(file_name)))
                .collect();

        if redistributable_paths.iter().all(|path| path.1.exists()) {
            return Ok(());
        }

        let loading_bar_type = crate::event::LoadingBarType::PluginDownload {
            plugin_name: self.get_name().clone(),
        };
        let loading_bar_total = 100.0;

        let loading_bar = crate::event::init_or_edit_loading(
            None,
            loading_bar_type,
            loading_bar_total,
            "Downloading packwiz redistributable",
        )
        .await?;

        log::info!("Downloading redistributable");
        for (key, path) in redistributable_paths.iter() {
            log::debug!("Downloading {}", key);

            if let Some(url) = Self::REDISTRIBUTABLE_LINKS.get(key) {
                let res = self
                    .download_file(
                        state,
                        &url,
                        path,
                        Some((
                            &loading_bar,
                            loading_bar_total / redistributable_paths.len() as f64,
                        )),
                    )
                    .await;

                if res.is_err() {
                    log::error!("Failed to download redistributable {}", key);
                    continue;
                }
            } else {
                log::error!("Url for {} not found", key);
                continue;
            }
        }

        Ok(())
    }

    async fn create_instance_from_pack(
        &self,
        pack: &PackwizPack,
        pack_path: &str,
    ) -> crate::Result<(String, PathBuf)> {
        let (mod_loader, mod_loader_version) = PackwizPlugin::extract_mod_loader(&pack.versions)?;

        let instance_id = instance_create(
            pack.name.to_string(),
            pack.versions.minecraft.to_string(),
            mod_loader,
            mod_loader_version,
            None,
            None,
        )
        .await?;

        log::info!("Installing pack in instance {:?}", instance_id);

        let instance_folder = get_dir(&instance_id).await?;

        self.save_settings(
            &instance_folder,
            &PackwizSettings {
                path: pack_path.to_string(),
            },
        )
        .await?;

        Ok((instance_id, instance_folder))
    }

    async fn update_pack(
        &self,
        instance_id: String,
        instance_folder: &PathBuf,
    ) -> crate::Result<()> {
        log::info!("Updating pack in instance {:?}", &instance_id);

        let settings = self
            .load_settings_from_instance_folder(&instance_folder)
            .await?;

        self.get_command_to_update_pack(&settings, &instance_folder)
            .await?
            .output()
            .await?;

        log::info!("Pack in instance {} updated successfully!", &instance_id);

        Ok(())
    }

    async fn init_command(&self, state: &LauncherState) -> crate::Result<()> {
        let plugin_dir = <dyn InstancePlugin>::get_plugin_dir(self).await?;

        if !plugin_dir.exists() || plugin_dir.metadata().is_err() {
            log::debug!("Creating plugin directory");
            create_dir_all(&plugin_dir).await?;
        }

        self.download_redistributable(&state).await?;

        Ok(())
    }

    async fn import_pack_command(&self, state: &LauncherState, path: String) -> crate::Result<()> {
        let packwiz_pack = self.get_pack_from_url_or_path(&state, &path).await?;

        let (instance_id, instance_folder) =
            self.create_instance_from_pack(&packwiz_pack, &path).await?;

        self.update_pack(instance_id, &instance_folder).await?;

        Ok(())
    }

    async fn update_pack_command(&self, instance_id: String) -> crate::Result<()> {
        let instance_folder = get_dir(&instance_id).await?;

        self.update_pack(instance_id, &instance_folder).await?;

        Ok(())
    }

    async fn clear_cache_command(&self) -> crate::Result<()> {
        let plugin_dir = <dyn InstancePlugin>::get_plugin_dir(self).await?;

        remove_dir_all(plugin_dir).await?;

        Ok(())
    }
}

#[async_trait]
impl InstancePlugin for PackwizPlugin {
    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_description(&self) -> String {
        self.description.clone()
    }

    async fn init(&self) -> crate::Result<()> {
        let state = LauncherState::get().await?;
        self.init_command(&state).await
    }

    async fn call(&self, data: &str) -> crate::Result<()> {
        let state = LauncherState::get().await?;

        let data = serde_json::from_str::<PackwizPluginData>(data)?;

        match data {
            PackwizPluginData::Import { url } => self.import_pack_command(&state, url).await?,
            PackwizPluginData::Update { instance_id } => {
                self.update_pack_command(instance_id).await?
            }
        }

        Ok(())
    }

    async fn unload(&self) -> crate::Result<()> {
        Ok(())
    }

    async fn clear_cache(&self) -> crate::Result<()> {
        self.clear_cache_command().await
    }
}
