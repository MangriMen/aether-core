use std::sync::Arc;

use crate::{
    features::{
        events::{EventEmitter, PluginEvent, PluginEventType},
        plugins::{Importer, ImportersRegistry, PluginError, PluginRegistry, PluginState},
    },
    shared::IoError,
};

pub struct PluginInfrastructureListener<E: EventEmitter, IR: ImportersRegistry> {
    plugin_registry: Arc<PluginRegistry<E>>,
    importers_registry: Arc<IR>,
}

impl<E: EventEmitter, IR: ImportersRegistry> PluginInfrastructureListener<E, IR> {
    pub fn new(plugin_registry: Arc<PluginRegistry<E>>, importers_registry: Arc<IR>) -> Self {
        Self {
            plugin_registry,
            importers_registry,
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

            let plugin = self.plugin_registry.get(&plugin_id)?;
            let state = plugin.state.clone();
            drop(plugin);

            let capabilities = self.plugin_registry.get_capabilities(&plugin_id)?;

            if matches!(state, PluginState::Loading) {
                return Ok(());
            }

            if let Some(capabilities) = capabilities {
                for capability in capabilities.importers {
                    match state {
                        crate::features::plugins::PluginState::Loaded(_) => {
                            self.importers_registry
                                .add(Importer {
                                    plugin_id: plugin_id.clone(),
                                    capability,
                                })
                                .await?;
                        }
                        crate::features::plugins::PluginState::NotLoaded
                        | crate::features::plugins::PluginState::Unloading
                        | crate::features::plugins::PluginState::Failed(_) => {
                            self.importers_registry.remove(&capability.id).await?;
                        }
                        _ => {}
                    }
                }
            }

            Ok(())
        }
        .await;

        if let Err(err) = result {
            log::error!("{}", err)
        }
    }
}
