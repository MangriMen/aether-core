use std::sync::Arc;

use crate::features::plugins::PluginRegistry;

use super::PluginDto;

pub struct ListPluginsDtoUseCase {
    plugin_registry: Arc<PluginRegistry>,
}

impl ListPluginsDtoUseCase {
    pub fn new(plugin_registry: Arc<PluginRegistry>) -> Self {
        Self { plugin_registry }
    }

    pub async fn execute(&self) -> crate::Result<Vec<PluginDto>> {
        Ok(self.plugin_registry.list().map(PluginDto::from).collect())
    }
}
