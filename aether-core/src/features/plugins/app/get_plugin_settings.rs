use crate::features::plugins::{PluginSettings, PluginSettingsStorage};

pub async fn get_plugin_settings<S>(storage: &S, plugin_id: &str) -> crate::Result<PluginSettings>
where
    S: PluginSettingsStorage + ?Sized,
{
    Ok(storage.get(plugin_id).await?.unwrap_or_default())
}
