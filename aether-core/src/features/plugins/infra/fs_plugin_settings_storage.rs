use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    features::{
        plugins::{PluginSettings, PluginSettingsStorage},
        settings::LocationInfo,
    },
    shared::{read_toml_async, write_toml_async, StorageError},
};

pub struct FsPluginSettingsStorage {
    location_info: Arc<LocationInfo>,
}

impl FsPluginSettingsStorage {
    pub async fn read<T>(&self, path: &Path) -> Result<Option<T>, StorageError>
    where
        T: DeserializeOwned,
    {
        if !path.exists() {
            return Ok(None);
        }

        let value = read_toml_async::<T>(path)
            .await
            .map_err(|err| StorageError::ReadError(err.to_string()))?;

        Ok(Some(value))
    }

    pub async fn write<T>(&self, path: &Path, value: &T) -> Result<(), StorageError>
    where
        T: Serialize,
    {
        write_toml_async(path, value)
            .await
            .map_err(|err| StorageError::WriteError(err.to_string()))
    }
}

impl FsPluginSettingsStorage {
    pub fn new(location_info: Arc<LocationInfo>) -> Self {
        Self { location_info }
    }

    fn get_plugin_settings_path(&self, plugin_id: &str) -> PathBuf {
        self.location_info.plugin_settings(plugin_id)
    }
}

#[async_trait]
impl PluginSettingsStorage for FsPluginSettingsStorage {
    async fn get(&self, plugin_id: &str) -> Result<Option<PluginSettings>, StorageError> {
        self.read(&self.get_plugin_settings_path(plugin_id)).await
    }

    async fn upsert(&self, plugin_id: &str, settings: &PluginSettings) -> Result<(), StorageError> {
        self.write(&self.get_plugin_settings_path(plugin_id), settings)
            .await
    }
}
