use std::sync::Arc;

use crate::features::{
    instance::{ImportConfig, InstanceError},
    plugins::{DefaultPluginInstanceFunctionsExt, PluginRegistry, PluginState},
};

pub struct ListImportConfigsUseCase {
    plugin_registry: Arc<PluginRegistry>,
}

impl ListImportConfigsUseCase {
    pub fn new(plugin_registry: Arc<PluginRegistry>) -> Self {
        Self { plugin_registry }
    }

    pub async fn execute(&self) -> Result<Vec<ImportConfig>, InstanceError> {
        self.get_plugin_import_configs().await
    }

    pub async fn get_plugin_import_configs(&self) -> Result<Vec<ImportConfig>, InstanceError> {
        let mut configs: Vec<ImportConfig> = Vec::new();

        for plugin in self.plugin_registry.list() {
            let PluginState::Loaded(instance) = &plugin.state else {
                continue;
            };

            let mut plugin_guard = instance.lock().await;

            if !plugin_guard.supports_get_import_config() {
                continue;
            }

            if let Ok(config) = plugin_guard.get_import_config() {
                configs.push(config);
            }
        }

        Ok(configs)
    }
}
