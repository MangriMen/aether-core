use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;

use crate::{
    features::{
        plugins::{LoadConfig, Plugin, PluginManifest, PluginStorage},
        settings::LocationInfo,
    },
    shared::{read_async, read_toml_async, sha1_async},
};

pub struct FsPluginStorage {
    location_info: Arc<LocationInfo>,
}

impl FsPluginStorage {
    pub fn new(location_info: Arc<LocationInfo>) -> Self {
        Self { location_info }
    }

    fn get_metadata_path(dir: &Path) -> PathBuf {
        dir.join("plugin.toml")
    }

    async fn load_manifest(dir: &Path) -> crate::Result<PluginManifest> {
        Ok(read_toml_async(&Self::get_metadata_path(dir)).await?)
    }

    async fn calc_hash(dir: &Path, manifest: &PluginManifest) -> crate::Result<String> {
        let relative_file_path = match manifest.load.clone() {
            LoadConfig::Extism { file, .. } => file,
            LoadConfig::Native { lib_path } => lib_path,
        };

        let absolute_file_path = dir.join(relative_file_path);
        let file_content = read_async(&absolute_file_path).await?;
        Ok(sha1_async(file_content).await?)
    }

    async fn load_from_dir(&self, dir: &Path) -> crate::Result<Plugin> {
        let manifest = Self::load_manifest(dir).await?;
        let hash = Self::calc_hash(dir, &manifest).await?;

        Ok(Plugin {
            manifest,
            hash,
            instance: None,
        })
    }
}

#[async_trait]
impl PluginStorage for FsPluginStorage {
    async fn list(&self) -> crate::Result<HashMap<String, Plugin>> {
        let plugins_dir = self.location_info.plugins_dir();

        if !plugins_dir.exists() {
            tokio::fs::create_dir_all(&plugins_dir).await?;
        }

        let mut dir_entries = tokio::fs::read_dir(&plugins_dir).await?;
        let mut plugins = HashMap::new();

        while let Some(dir_entry) = dir_entries.next_entry().await? {
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

    async fn get(&self, plugin_id: &str) -> crate::Result<Plugin> {
        let plugin_dir = self.location_info.plugin_dir(plugin_id);
        self.load_from_dir(&plugin_dir).await
    }
}
