use std::path::PathBuf;

use async_trait::async_trait;

use crate::features::plugins::{PluginSettings, PluginSettingsManager, PluginSettingsStorage};

use super::EditPluginSettings;

pub struct PluginSettingsManagerImpl<PSS>
where
    PSS: PluginSettingsStorage + Send + Sync,
{
    storage: PSS,
}

impl<PSS> PluginSettingsManagerImpl<PSS>
where
    PSS: PluginSettingsStorage + Send + Sync,
{
    pub fn new(storage: PSS) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl<PSS> PluginSettingsManager for PluginSettingsManagerImpl<PSS>
where
    PSS: PluginSettingsStorage + Send + Sync,
{
    async fn get(&self, plugin_id: &str) -> crate::Result<Option<PluginSettings>> {
        Ok(self.storage.get(plugin_id).await?)
    }

    async fn upsert(&self, plugin_id: &str, settings: &PluginSettings) -> crate::Result<()> {
        Ok(self.storage.upsert(plugin_id, settings).await?)
    }

    async fn edit(&self, plugin_id: &str, edit_settings: &EditPluginSettings) -> crate::Result<()> {
        let current = self.storage.get(plugin_id).await?.unwrap_or_default();
        let merged = apply_edit_changes(current, edit_settings);
        Ok(self.storage.upsert(plugin_id, &merged).await?)
    }
}

fn apply_edit_changes(
    mut settings: PluginSettings,
    edit_settings: &EditPluginSettings,
) -> PluginSettings {
    if let Some(allowed_hosts) = &edit_settings.allowed_hosts {
        settings.allowed_hosts = allowed_hosts.clone();
    }

    if let Some(allowed_paths) = &edit_settings.allowed_paths {
        let filtered = allowed_paths
            .iter()
            .filter(|(from, _)| PathBuf::from(from).exists())
            .cloned()
            .collect();

        settings.allowed_paths = filtered;
    }

    settings
}
