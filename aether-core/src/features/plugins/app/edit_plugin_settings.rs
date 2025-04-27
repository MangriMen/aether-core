use std::path::PathBuf;

use crate::features::plugins::{PluginSettings, PluginSettingsStorage};

use super::EditPluginSettings;

pub async fn edit_plugin_settings<S>(
    storage: &S,
    plugin_id: &str,
    edit_settings: &EditPluginSettings,
) -> crate::Result<()>
where
    S: PluginSettingsStorage + ?Sized,
{
    let current = storage.get(plugin_id).await?.unwrap_or_default();
    let merged = apply_edit_changes(current, edit_settings);
    Ok(storage.upsert(plugin_id, &merged).await?)
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
