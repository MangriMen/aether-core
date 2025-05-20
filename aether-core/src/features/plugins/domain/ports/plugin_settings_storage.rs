use async_trait::async_trait;

use crate::{features::plugins::PluginSettings, shared::StorageError};

#[async_trait]
pub trait PluginSettingsStorage: Send + Sync {
    async fn get(&self, plugin_id: &str) -> Result<Option<PluginSettings>, StorageError>;
    async fn upsert(&self, plugin_id: &str, settings: &PluginSettings) -> Result<(), StorageError>;
}
