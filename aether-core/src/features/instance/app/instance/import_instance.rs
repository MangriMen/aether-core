use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::features::{
    events::EventEmitter,
    instance::InstanceError,
    plugins::{
        DefaultPluginInstanceFunctionsExt, PluginImportInstance, PluginRegistry, PluginState,
    },
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImportInstance {
    pub plugin_id: String,
    pub importer_id: String,
    pub path: String,
}

pub struct ImportInstanceUseCase<E: EventEmitter> {
    plugin_registry: Arc<PluginRegistry<E>>,
}

impl<E: EventEmitter> ImportInstanceUseCase<E> {
    pub fn new(plugin_registry: Arc<PluginRegistry<E>>) -> Self {
        Self { plugin_registry }
    }

    pub async fn execute(&self, import_instance: ImportInstance) -> Result<(), InstanceError> {
        let ImportInstance {
            plugin_id,
            importer_id,
            path,
        } = import_instance;

        self.import_by_plugin(&plugin_id, importer_id, path).await?;

        Ok(())
    }

    pub async fn import_by_plugin(
        &self,
        plugin_id: &str,
        importer_id: String,
        path: String,
    ) -> Result<bool, InstanceError> {
        let plugin = self.plugin_registry.get(plugin_id).map_err(|_| {
            InstanceError::InstanceImportError {
                plugin_id: plugin_id.to_owned(),
                err: "Unsupported pack type".to_owned(),
            }
        })?;

        let PluginState::Loaded(instance) = &plugin.state else {
            return Err(InstanceError::InstanceImportError {
                plugin_id: plugin_id.to_owned(),
                err: "Plugin disabled".to_owned(),
            });
        };

        let mut plugin_guard = instance.lock().await;

        plugin_guard
            .import(PluginImportInstance { importer_id, path })
            .map_err(|e| InstanceError::InstanceImportError {
                plugin_id: plugin_id.to_owned(),
                err: e.to_string(),
            })
    }
}
