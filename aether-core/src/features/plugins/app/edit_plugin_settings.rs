use std::{path::PathBuf, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::features::plugins::{PathMapping, PluginError, PluginSettings, PluginSettingsStorage};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct EditPluginSettings {
    pub allowed_hosts: Option<Vec<String>>,
    pub allowed_paths: Option<Vec<PathMapping>>,
}

pub struct EditPluginSettingsUseCase<PSS: PluginSettingsStorage> {
    plugin_settings_storage: Arc<PSS>,
}

impl<PSS: PluginSettingsStorage> EditPluginSettingsUseCase<PSS> {
    pub fn new(plugin_settings_storage: Arc<PSS>) -> Self {
        Self {
            plugin_settings_storage,
        }
    }

    pub async fn execute(
        &self,
        plugin_id: String,
        edit_settings: EditPluginSettings,
    ) -> Result<(), PluginError> {
        let current = self
            .plugin_settings_storage
            .get(&plugin_id)
            .await?
            .unwrap_or_default();
        let merged = apply_edit_changes(current, &edit_settings);

        self.plugin_settings_storage
            .upsert(&plugin_id, &merged)
            .await
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
