use std::{path::PathBuf, sync::Arc};

use path_slash::PathBufExt;
use serde::{Deserialize, Serialize};

use crate::features::{
    events::EventEmitter,
    instance::InstanceError,
    plugins::{
        CapabilityRegistry, ImporterCapability, PluginImportInstance, PluginInstanceExt,
        PluginRegistry, PluginState,
    },
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImportInstance {
    pub plugin_id: String,
    pub importer_id: String,
    pub path: String,
}

pub struct ImportInstanceUseCase<E: EventEmitter, IR: CapabilityRegistry<ImporterCapability>> {
    plugin_registry: Arc<PluginRegistry<E>>,
    importers_registry: Arc<IR>,
}

impl<E: EventEmitter, IR: CapabilityRegistry<ImporterCapability>> ImportInstanceUseCase<E, IR> {
    pub fn new(plugin_registry: Arc<PluginRegistry<E>>, importers_registry: Arc<IR>) -> Self {
        Self {
            plugin_registry,
            importers_registry,
        }
    }

    pub async fn execute(&self, import_instance: ImportInstance) -> Result<(), InstanceError> {
        let ImportInstance {
            plugin_id,
            importer_id,
            path,
        } = import_instance;

        self.import_by_plugin(plugin_id, importer_id, &path).await?;

        Ok(())
    }

    pub async fn import_by_plugin(
        &self,
        plugin_id: String,
        importer_id: String,
        path: &str,
    ) -> Result<(), InstanceError> {
        let capability_entry = self
            .importers_registry
            .find_by_plugin_and_capability_id(plugin_id.clone(), importer_id.clone())
            .await
            .map_err(|_| InstanceError::ImporterNotFound {
                importer_id: importer_id.to_owned(),
            })?;

        let plugin = self.plugin_registry.get(&plugin_id).map_err(|err| {
            tracing::debug!("Error importing instance (plugin not found): {:?}", err);

            InstanceError::ImporterNotFound {
                importer_id: importer_id.to_owned(),
            }
        })?;

        let PluginState::Loaded(instance) = &plugin.state else {
            tracing::debug!("Error importing instance (plugin disabled)");

            return Err(InstanceError::ImporterNotFound {
                importer_id: importer_id.to_owned(),
            });
        };

        let mut plugin_guard = instance.lock().await;

        let import_handler = &capability_entry.capability.handler;

        if !plugin_guard.supports(import_handler) {
            tracing::debug!("Error importing instance (plugin doesn't supports import)");

            return Err(InstanceError::ImporterNotFound {
                importer_id: importer_id.to_owned(),
            });
        }

        plugin_guard
            .call(
                import_handler,
                PluginImportInstance {
                    importer_id: importer_id.to_owned(),
                    path: PathBuf::from(path).to_slash_lossy().to_string(),
                },
            )
            .map_err(|err| {
                tracing::debug!("Error importing instance: {:?}", err);
                InstanceError::ImportFailed {
                    importer_id: importer_id.to_owned(),
                }
            })
    }
}
