use std::sync::Arc;

use crate::features::plugins::{PluginManifest, PluginRegistry};

pub struct GetPluginManifestUseCase {
    plugin_registry: Arc<PluginRegistry>,
}

impl GetPluginManifestUseCase {
    pub fn new(plugin_registry: Arc<PluginRegistry>) -> Self {
        Self { plugin_registry }
    }
    pub async fn execute(&self, plugin_id: String) -> crate::Result<PluginManifest> {
        self.plugin_registry.get_manifest(&plugin_id)
    }
}
