use std::sync::Arc;

use crate::features::{
    events::EventEmitter,
    instance::{
        InstanceError, InstanceInstallStage, InstanceStorage, InstanceStorageExt, PackInfo,
    },
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

        let importer = self
            .importers_registry
            .get(&modpack_id)
            .await
            .map_err(|_| InstanceError::UpdaterNotFound {
                modpack_id: modpack_id.clone(),
            })?;

        let plugin_id = &importer.plugin_id;

        let plugin = self.plugin_registry.get(plugin_id).map_err(|err| {
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

        if !plugin_guard.supports_update() {
            tracing::debug!("Error updating instance (plugin doesn't supports update)");

            return Err(InstanceError::UpdaterNotFound {
                modpack_id: modpack_id.clone(),
            });
        }

        plugin_guard.update(instance_id).map_err(|err| {
            tracing::debug!("Error updating instance: {:?}", err);
            InstanceError::UpdateFailed {
                modpack_id: modpack_id.clone(),
            }
        })
    }
}
