use std::sync::Arc;

use crate::features::plugins::{PluginSettings, PluginSettingsStorage};

pub struct GetPluginSettingsUseCase<PSS: PluginSettingsStorage> {
    storage: Arc<PSS>,
}

impl<PSS: PluginSettingsStorage> GetPluginSettingsUseCase<PSS> {
    pub fn new(plugin_settings_storage: Arc<PSS>) -> Self {
        Self {
            storage: plugin_settings_storage,
        }
    }

    pub async fn execute(&self, plugin_id: String) -> crate::Result<Option<PluginSettings>> {
        Ok(self.storage.get(&plugin_id).await?)
    }
}
