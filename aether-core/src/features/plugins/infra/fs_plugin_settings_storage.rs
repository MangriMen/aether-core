use std::path::PathBuf;

use async_trait::async_trait;

use crate::{
    core::LauncherState,
    features::plugins::{PluginSettings, PluginSettingsStorage},
    utils::io::{read_toml_async, write_toml_async},
};

pub struct FsPluginSettingsStorage;

impl FsPluginSettingsStorage {
    fn get_plugin_settings_path(state: &LauncherState, plugin_id: &str) -> PathBuf {
        state.locations.plugin_settings(plugin_id)
    }
}

#[async_trait]
impl PluginSettingsStorage for FsPluginSettingsStorage {
    async fn get(
        &self,
        state: &LauncherState,
        plugin_id: &str,
    ) -> crate::Result<Option<PluginSettings>> {
        let path = Self::get_plugin_settings_path(state, plugin_id);

        if !path.exists() {
            return Ok(None);
        }

        Ok(Some(read_toml_async(path).await?))
    }

    async fn upsert(
        &self,
        state: &LauncherState,
        plugin_id: &str,
        settings: &PluginSettings,
    ) -> crate::Result<()> {
        let path = Self::get_plugin_settings_path(state, plugin_id);

        write_toml_async(path, settings).await?;

        Ok(())
    }
}
