use std::sync::Arc;

use crate::features::{
    events::EventEmitter,
    instance::InstanceError,
    plugins::{PluginImporters, PluginRegistry},
};

pub struct ListImportersUseCase<E: EventEmitter> {
    plugin_registry: Arc<PluginRegistry<E>>,
}

impl<E: EventEmitter> ListImportersUseCase<E> {
    pub fn new(plugin_registry: Arc<PluginRegistry<E>>) -> Self {
        Self { plugin_registry }
    }

    pub async fn execute(&self) -> Result<Vec<PluginImporters>, InstanceError> {
        let plugins_ids = self.plugin_registry.get_enabled_ids();

        Ok(plugins_ids
            .iter()
            .filter_map(|plugin_id| {
                let capabilities = self.plugin_registry.get_capabilities(plugin_id);

                match capabilities {
                    Ok(capabilities) => capabilities.map(|capabilities| PluginImporters {
                        plugin_id: plugin_id.to_owned(),
                        importers: capabilities.importers,
                    }),
                    Err(_) => None,
                }
            })
            .collect())
    }
}
