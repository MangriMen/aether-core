use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::features::{
    instance::InstanceError,
    plugins::{DefaultPluginInstanceFunctionsExt, PluginRegistry},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImportInstance {
    pub path_or_url: String,
    pub plugin_id: Option<String>,
}

pub struct ImportInstanceUseCase {
    plugin_registry: Arc<PluginRegistry>,
}

impl ImportInstanceUseCase {
    pub fn new(plugin_registry: Arc<PluginRegistry>) -> Self {
        Self { plugin_registry }
    }

    pub async fn execute(&self, import_instance: ImportInstance) -> Result<(), InstanceError> {
        let ImportInstance {
            path_or_url,
            plugin_id,
        } = import_instance;

        if let Some(plugin_id) = plugin_id {
            self.import_by_plugin(&path_or_url, &plugin_id).await?;
        }

        Ok(())
    }

    pub async fn import_by_plugin(
        &self,
        path_or_url: &str,
        plugin_id: &str,
    ) -> Result<bool, InstanceError> {
        let plugin = self
            .plugin_registry
            .get(plugin_id)
            .map_err(|_| InstanceError::InstanceImportError("Unsupported pack type".to_owned()))?;

        let Some(instance) = &plugin.instance else {
            return Err(InstanceError::InstanceImportError(
                "Plugin disabled".to_owned(),
            ));
        };

        let mut plugin_guard = instance.lock().await;

        plugin_guard.import(path_or_url).map_err(|_| {
            InstanceError::InstanceImportError(format!(
                "Failed to import instance from plugin {plugin_id}"
            ))
        })
    }
}
