use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::features::{
    instance::{Importer, ImporterCapability, InstanceError},
    plugins::{PluginImportInstance, PluginInstance, PluginInstanceExt},
};

pub struct PluginImporterProxy {
    instance: Arc<Mutex<dyn PluginInstance>>,
    capability: ImporterCapability,
}

impl PluginImporterProxy {
    pub fn new(instance: Arc<Mutex<dyn PluginInstance>>, capability: ImporterCapability) -> Self {
        Self {
            instance,
            capability,
        }
    }
}

#[async_trait]
impl Importer for PluginImporterProxy {
    fn info(&self) -> &ImporterCapability {
        &self.capability
    }

    async fn import(&self, path: &str) -> Result<(), InstanceError> {
        let mut plugin = self.instance.lock().await;
        let plugin_id = plugin.get_id();

        if !plugin.supports(&self.capability.handler) {
            tracing::error!(
                "Plugin '{}' promised handler '{}' for capability '{}', but function not found",
                plugin_id.clone(),
                self.capability.handler,
                self.capability.id
            );

            return Err(InstanceError::ImportFailed {
                importer_id: self.capability.id.clone(),
            });
        }

        plugin
            .call(
                &self.capability.handler,
                PluginImportInstance {
                    importer_id: self.capability.id.clone(),
                    path: path.to_owned(),
                },
            )
            .map_err(|err| {
                tracing::error!(
                    "Error importing instance by plugin '{}': {:?}",
                    plugin_id,
                    err
                );

                InstanceError::ImportFailed {
                    importer_id: self.capability.id.clone(),
                }
            })
    }
}
