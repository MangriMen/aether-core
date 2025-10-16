use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use log::debug;

use crate::{
    features::{
        plugins::{LoadConfig, Plugin, PluginError, PluginManifest, PluginState, PluginStorage},
        settings::LocationInfo,
    },
    shared::{create_dir_all, read_async, read_dir, read_toml_async, sha1_async, IoError},
};

pub struct FsPluginStorage {
    location_info: Arc<LocationInfo>,
}

impl FsPluginStorage {
    pub fn new(location_info: Arc<LocationInfo>) -> Self {
        Self { location_info }
    }

    fn get_manifest_path(dir: &Path) -> PathBuf {
        dir.join("manifest.toml")
    }

    async fn load_manifest(dir: &Path) -> Result<PluginManifest, PluginError> {
        Ok(read_toml_async(&Self::get_manifest_path(dir)).await?)
    }

    async fn calc_hash(dir: &Path, manifest: &PluginManifest) -> Result<String, PluginError> {
        let relative_file_path = match manifest.load.clone() {
            LoadConfig::Extism { file, .. } => file,
            LoadConfig::Native { lib_path } => lib_path,
        };

        let absolute_file_path = dir.join(relative_file_path);
        let file_content = read_async(&absolute_file_path).await?;

        sha1_async(file_content).await.map_err(|error| {
            debug!("Failed to compute sha1: {error}");
            PluginError::HashConstructError
        })
    }

    async fn load_from_dir(&self, dir: &Path) -> Result<Plugin, PluginError> {
        let manifest = Self::load_manifest(dir).await?;
        let hash = Self::calc_hash(dir, &manifest).await?;

        Ok(Plugin {
            manifest,
            hash,
            state: PluginState::NotLoaded,
        })
    }
}

#[async_trait]
impl PluginStorage for FsPluginStorage {
    async fn list(&self) -> Result<HashMap<String, Plugin>, PluginError> {
        let plugins_dir = self.location_info.plugins_dir();

        if !plugins_dir.exists() {
            create_dir_all(&plugins_dir).await?;
        }

        let mut dir_entries = read_dir(&plugins_dir).await?;
        let mut plugins = HashMap::new();

        while let Some(dir_entry) = dir_entries.next_entry().await.map_err(IoError::from)? {
            let plugin_dir = dir_entry.path();

            match self.load_from_dir(&plugin_dir).await {
                Ok(plugin) => {
                    plugins.insert(plugin.manifest.metadata.id.clone(), plugin);
                }
                Err(e) => {
                    log::debug!(
                        "Failed to load plugin from '{}': {}",
                        plugin_dir.display(),
                        e
                    );
                }
            }
        }

        Ok(plugins)
    }

    async fn get(&self, plugin_id: &str) -> Result<Plugin, PluginError> {
        let plugin_dir = self.location_info.plugin_dir(plugin_id);
        self.load_from_dir(&plugin_dir).await
    }
}
