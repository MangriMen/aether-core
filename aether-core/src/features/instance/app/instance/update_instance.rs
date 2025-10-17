use std::sync::Arc;

use crate::features::{
    events::EventEmitter,
    instance::{InstanceError, InstanceStorage},
    plugins::{DefaultPluginInstanceFunctionsExt, PluginRegistry, PluginState},
};

pub struct UpdateInstanceUseCase<IS: InstanceStorage, E: EventEmitter> {
    instance_storage: Arc<IS>,
    plugin_registry: Arc<PluginRegistry<E>>,
}

impl<IS: InstanceStorage, E: EventEmitter> UpdateInstanceUseCase<IS, E> {
    pub fn new(instance_storage: Arc<IS>, plugin_registry: Arc<PluginRegistry<E>>) -> Self {
        Self {
            instance_storage,
            plugin_registry,
        }
    }

    pub async fn execute(&self, instance_id: String) -> Result<(), InstanceError> {
        let instance = self.instance_storage.get(&instance_id).await?;

        let Some(pack_info) = instance.pack_info else {
            return Err(InstanceError::InstanceUpdateError(
                "There is not pack info".to_owned(),
            ));
        };

        if let Some(plugin_id) = pack_info.plugin_id {
            self.update_by_plugin(&instance_id, &plugin_id).await
        } else {
            Ok(())
        }
    }

    pub async fn update_by_plugin(
        &self,
        instance_id: &str,
        plugin_id: &str,
    ) -> Result<(), InstanceError> {
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
