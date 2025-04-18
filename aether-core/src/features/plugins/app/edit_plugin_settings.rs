use std::path::PathBuf;

use crate::features::plugins::{PluginSettings, PluginSettingsStorage};

pub async fn edit_plugin_settings<S>(
    storage: &S,
    plugin_id: &str,
    new_settings: &PluginSettings,
) -> crate::Result<()>
where
    S: PluginSettingsStorage + ?Sized,
{
    let current = storage.get(plugin_id).await?.unwrap_or_default();
    let merged = merge_plugin_settings(current, new_settings);
    Ok(storage.upsert(plugin_id, &merged).await?)
}

fn merge_plugin_settings(mut base: PluginSettings, patch: &PluginSettings) -> PluginSettings {
    if let Some(allowed_hosts) = &patch.allowed_hosts {
        base.allowed_hosts = Some(allowed_hosts.clone());
    }

    if let Some(allowed_paths) = &patch.allowed_paths {
        let filtered = allowed_paths
            .iter()
            .filter(|(from, _)| PathBuf::from(from).exists())
            .cloned()
            .collect();

        base.allowed_paths = Some(filtered);
    }

    base
}
