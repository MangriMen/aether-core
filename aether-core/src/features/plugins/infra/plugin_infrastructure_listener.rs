use std::sync::Arc;

use crate::{
    features::{
        events::{EventEmitter, PluginEvent, PluginEventType},
        plugins::{
            CapabilityRegistry, ImporterCapability, PluginCapabilities, PluginError,
            PluginRegistry, PluginState, UpdaterCapability,
        },
    },
    shared::IoError,
};

pub struct PluginInfrastructureListener<
    E: EventEmitter,
    IR: CapabilityRegistry<ImporterCapability>,
    UR: CapabilityRegistry<UpdaterCapability>,
> {
    plugin_registry: Arc<PluginRegistry<E>>,
    importers_registry: Arc<IR>,
    updaters_registry: Arc<UR>,
}

impl<
        E: EventEmitter,
        IR: CapabilityRegistry<ImporterCapability>,
        UR: CapabilityRegistry<UpdaterCapability>,
    > PluginInfrastructureListener<E, IR, UR>
{
    pub fn new(
        plugin_registry: Arc<PluginRegistry<E>>,
        importers_registry: Arc<IR>,
        updaters_registry: Arc<UR>,
    ) -> Self {
        Self {
            plugin_registry,
            importers_registry,
            updaters_registry,
        }
    }

    pub async fn on_plugin_event(&self, data: String) {
        let result: Result<(), PluginError> = async {
            let plugin_event = serde_json::from_str::<PluginEvent>(&data)
                .map_err(|e| IoError::DeserializationError(e.to_string()))?;

            let plugin_id = match plugin_event.event {
                PluginEventType::Edit { plugin_id } => plugin_id,
                _ => return Ok(()),
            };

            let state = {
                let plugin = self.plugin_registry.get(&plugin_id)?;
                plugin.state.clone()
            };

            // Skip processing while plugin still loading
            if matches!(state, PluginState::Loading) {
                return Ok(());
            }

            let capabilities = self.plugin_registry.get_capabilities(&plugin_id)?;

            if let Some(capabilities) = capabilities {
                match state {
                    crate::features::plugins::PluginState::Loaded(_) => {
                        self.register_capabilities(plugin_id, &capabilities).await?;
                    }
                    crate::features::plugins::PluginState::NotLoaded
                    | crate::features::plugins::PluginState::Unloading
                    | crate::features::plugins::PluginState::Failed(_) => {
                        self.unregister_capabilities(plugin_id, &capabilities)
                            .await?;
                    }
                    _ => {}
                }
            }

            Ok(())
        }
        .await;

        if let Err(err) = result {
            log::error!("{}", err)
        }
    }

    async fn register_capabilities(
        &self,
        plugin_id: String,
        capabilities: &PluginCapabilities,
    ) -> Result<(), PluginError> {
        for capability in &capabilities.importers {
            self.importers_registry
                .add(
                    plugin_id.clone(),
                    capability.id.to_owned(),
                    capability.clone(),
                )
                .await?;
        }

        for capability in &capabilities.updaters {
            self.updaters_registry
                .add(
                    plugin_id.clone(),
                    capability.id.to_owned(),
                    capability.clone(),
                )
                .await?;
        }

        Ok(())
    }

    async fn unregister_capabilities(
        &self,
        plugin_id: String,
        capabilities: &PluginCapabilities,
    ) -> Result<(), PluginError> {
        for capability in &capabilities.importers {
            self.importers_registry
                .remove_by_plugin_and_capability(plugin_id.clone(), capability.id.to_owned())
                .await?;
        }

        for capability in &capabilities.updaters {
            self.updaters_registry
                .remove_by_plugin_and_capability(plugin_id.clone(), capability.id.to_owned())
                .await?;
        }

        Ok(())
    }
}
