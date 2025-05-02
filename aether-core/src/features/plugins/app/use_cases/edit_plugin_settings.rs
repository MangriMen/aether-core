use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;

use crate::{
    features::plugins::{EditPluginSettings, PluginSettings, PluginSettingsStorage},
    shared::domain::AsyncUseCaseWithInputAndError,
};

pub struct EditPluginSettingsUseCase<PSS: PluginSettingsStorage> {
    plugin_settings_storage: Arc<PSS>,
}

impl<PSS: PluginSettingsStorage> EditPluginSettingsUseCase<PSS> {
    pub fn new(plugin_settings_storage: Arc<PSS>) -> Self {
        Self {
            plugin_settings_storage,
        }
    }
}

#[async_trait]
impl<PSS> AsyncUseCaseWithInputAndError for EditPluginSettingsUseCase<PSS>
where
    PSS: PluginSettingsStorage + Send + Sync,
{
    type Input = (String, EditPluginSettings);
    type Output = ();
    type Error = crate::Error;

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let (plugin_id, edit_settings) = input;

        let current = self
            .plugin_settings_storage
            .get(&plugin_id)
            .await?
            .unwrap_or_default();
        let merged = apply_edit_changes(current, &edit_settings);
        Ok(self
            .plugin_settings_storage
            .upsert(&plugin_id, &merged)
            .await?)
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
