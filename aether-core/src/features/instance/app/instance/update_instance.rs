use std::sync::Arc;

use crate::features::{
    events::EventEmitter,
    instance::{InstanceError, InstanceStorage, PackInfo},
    plugins::{DefaultPluginInstanceFunctionsExt, ImportersRegistry, PluginRegistry, PluginState},
};

pub struct UpdateInstanceUseCase<IS: InstanceStorage, E: EventEmitter, IR: ImportersRegistry> {
    instance_storage: Arc<IS>,
    plugin_registry: Arc<PluginRegistry<E>>,
    importers_registry: Arc<IR>,
}

impl<IS: InstanceStorage, E: EventEmitter, IR: ImportersRegistry> UpdateInstanceUseCase<IS, E, IR> {
    pub fn new(
        instance_storage: Arc<IS>,
        plugin_registry: Arc<PluginRegistry<E>>,
        importers_registry: Arc<IR>,
    ) -> Self {
        Self {
            instance_storage,
            plugin_registry,
            importers_registry,
        }
    }

    pub async fn execute(&self, instance_id: String) -> Result<(), InstanceError> {
        let instance = self.instance_storage.get(&instance_id).await?;

        let Some(pack_info) = instance.pack_info else {
            return Err(InstanceError::InstanceUpdateError(
                "There is no pack info".to_owned(),
            ));
        };

        self.update_by_plugin(&instance_id, &pack_info).await
    }

    pub async fn update_by_plugin(
        &self,
        instance_id: &str,
        pack_info: &PackInfo,
    ) -> Result<(), InstanceError> {
        let importer = self
            .importers_registry
            .get(&pack_info.modpack_id)
            .await
            .map_err(|_| InstanceError::InstanceImportError {
                plugin_id: "unknown".to_owned(),
                err: "Importer not found".to_owned(),
            })?;

        let plugin_id = &importer.plugin_id;

        let plugin = self
            .plugin_registry
            .get(plugin_id)
            .map_err(|_| InstanceError::InstanceUpdateError("Unsupported pack type".to_owned()))?;

        let PluginState::Loaded(plugin_instance) = &plugin.state else {
            return Err(InstanceError::InstanceUpdateError(format!(
                "Can't get plugin \"{}\" to update instance. Check if it is installed and enabled",
                &plugin_id
            )));
        };

        let mut plugin_guard = plugin_instance.lock().await;

        if !plugin_guard.supports_update() {
            return Err(InstanceError::InstanceUpdateError(format!(
                "Plugin \"{}\" doesn't supports update",
                &plugin_id
            )));
        }

        plugin_guard.update(instance_id).map_err(|_| {
            InstanceError::InstanceUpdateError(format!(
                "Failed to update instance with plugin {}",
                &plugin_id
            ))
        })
    }
}
