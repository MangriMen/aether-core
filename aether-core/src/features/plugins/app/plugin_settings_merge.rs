use std::path::PathBuf;

use crate::features::plugins::PluginSettings;

pub fn merge_plugin_settings(mut base: PluginSettings, patch: &PluginSettings) -> PluginSettings {
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
