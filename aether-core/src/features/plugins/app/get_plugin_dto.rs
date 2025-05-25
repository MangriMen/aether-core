use std::sync::Arc;

use crate::features::plugins::PluginRegistry;

use super::PluginDto;

pub struct GetPluginDtoUseCase {
    plugin_registry: Arc<PluginRegistry>,
}

impl GetPluginDtoUseCase {
    pub fn new(plugin_registry: Arc<PluginRegistry>) -> Self {
        Self { plugin_registry }
    }
    pub async fn execute(&self, plugin_id: String) -> crate::Result<PluginDto> {
        self.plugin_registry.get(&plugin_id).map(PluginDto::from)
    }
}
