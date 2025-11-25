use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::features::{
    events::EventEmitter,
    instance::InstanceError,
    plugins::{
        DefaultPluginInstanceFunctionsExt, ImportersRegistry, PluginImportInstance, PluginRegistry,
        PluginState,
    },
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImportInstance {
    pub importer_id: String,
    pub path: String,
}

pub struct ImportInstanceUseCase<E: EventEmitter, IR: ImportersRegistry> {
    plugin_registry: Arc<PluginRegistry<E>>,
    importers_registry: Arc<IR>,
}

impl<E: EventEmitter, IR: ImportersRegistry> ImportInstanceUseCase<E, IR> {
    pub fn new(plugin_registry: Arc<PluginRegistry<E>>, importers_registry: Arc<IR>) -> Self {
        Self {
            plugin_registry,
            importers_registry,
        }
    }

    pub async fn execute(&self, import_instance: ImportInstance) -> Result<(), InstanceError> {
        let ImportInstance { importer_id, path } = import_instance;

        self.import_by_plugin(&importer_id, &path).await?;

        Ok(())
    }

    pub async fn import_by_plugin(
        &self,
        importer_id: &str,
        path: &str,
    ) -> Result<(), InstanceError> {
        let importer = self
            .importers_registry
            .get(importer_id)
            .await
            .map_err(|_| InstanceError::InstanceImportError {
                plugin_id: "unknown".to_owned(),
                err: "Importer not found".to_owned(),
            })?;

        let plugin_id = &importer.plugin_id;

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
            .import(PluginImportInstance {
                importer_id: importer_id.to_owned(),
                path: path.to_owned(),
            })
            .map_err(|e| InstanceError::InstanceImportError {
                plugin_id: plugin_id.to_owned(),
                err: e.to_string(),
            })
    }
}
