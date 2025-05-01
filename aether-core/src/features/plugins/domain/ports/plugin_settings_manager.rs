use async_trait::async_trait;

use crate::features::plugins::{EditPluginSettings, PluginSettings};

#[async_trait]
pub trait PluginSettingsManager {
    async fn get(&self, plugin_id: &str) -> crate::Result<Option<PluginSettings>>;
    async fn upsert(&self, plugin_id: &str, settings: &PluginSettings) -> crate::Result<()>;
    async fn edit(&self, plugin_id: &str, settings: &EditPluginSettings) -> crate::Result<()>;
}
