use std::sync::Arc;

use crate::features::{
    events::EventEmitter,
    instance::{
        InstanceError, InstanceInstallStage, InstanceStorage, InstanceStorageExt, PackInfo,
    },
    plugins::{
        CapabilityRegistry, PluginInstanceExt, PluginRegistry, PluginState, UpdaterCapability,
    },
};

pub struct UpdateInstanceUseCase<
    IS: InstanceStorage,
    E: EventEmitter,
    UR: CapabilityRegistry<UpdaterCapability>,
> {
    instance_storage: Arc<IS>,
    plugin_registry: Arc<PluginRegistry<E>>,
    importers_registry: Arc<UR>,
}

impl<IS: InstanceStorage, E: EventEmitter, UR: CapabilityRegistry<UpdaterCapability>>
    UpdateInstanceUseCase<IS, E, UR>
{
    pub fn new(
        instance_storage: Arc<IS>,
        plugin_registry: Arc<PluginRegistry<E>>,
        importers_registry: Arc<UR>,
    ) -> Self {
        Self {
            instance_storage,
            plugin_registry,
            importers_registry,
        }
    }

    pub async fn execute(&self, instance_id: String) -> Result<(), InstanceError> {
        let original_stage = self.instance_storage.get(&instance_id).await?.install_stage;

        let result = self.perform_update(&instance_id).await;

        self.instance_storage
            .upsert_with(&instance_id, |instance| {
                instance.install_stage = match result {
                    Ok(_) => InstanceInstallStage::Installed,
                    Err(_) => original_stage,
                };
                Ok(())
            })
            .await?;

        result
    }

    pub async fn perform_update(&self, instance_id: &str) -> Result<(), InstanceError> {
        self.instance_storage
            .upsert_with(instance_id, |instance| {
                instance.install_stage = InstanceInstallStage::PackInstalling;
                Ok(())
            })
            .await?;

        let instance = self.instance_storage.get(instance_id).await?;

        let Some(pack_info) = instance.pack_info else {
            return Err(InstanceError::PackInfoNotFound);
        };

        self.update_by_plugin(instance_id, &pack_info).await
    }

    pub async fn update_by_plugin(
        &self,
        instance_id: &str,
        pack_info: &PackInfo,
    ) -> Result<(), InstanceError> {
        let modpack_id = pack_info.modpack_id.clone();
        let plugin_id = pack_info.plugin_id.clone();

        let capability_entry = self
            .importers_registry
            .find_by_plugin_and_capability_id(plugin_id.clone(), modpack_id.clone())
            .await
            .map_err(|_| InstanceError::UpdaterNotFound {
                modpack_id: modpack_id.clone(),
            })?;

        let plugin = self.plugin_registry.get(&plugin_id).map_err(|err| {
            tracing::debug!("Error updating instance (plugin not found): {:?}", err);

            InstanceError::UpdaterNotFound {
                modpack_id: modpack_id.clone(),
            }
        })?;

        let PluginState::Loaded(plugin_instance) = &plugin.state else {
            tracing::debug!("Error updating instance (plugin disabled)");

            return Err(InstanceError::UpdaterNotFound {
                modpack_id: modpack_id.clone(),
            });
        };

        let mut plugin_guard = plugin_instance.lock().await;

        let update_handler = &capability_entry.capability.handler;

        if !plugin_guard.supports(update_handler) {
            tracing::debug!("Error updating instance (plugin doesn't supports update)");

            return Err(InstanceError::UpdaterNotFound {
                modpack_id: modpack_id.clone(),
            });
        }

        plugin_guard
            .call(update_handler, instance_id)
            .map_err(|err| {
                tracing::debug!("Error updating instance: {:?}", err);
                InstanceError::UpdateFailed {
                    modpack_id: modpack_id.clone(),
                }
            })
    }
}
