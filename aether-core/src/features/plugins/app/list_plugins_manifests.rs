use std::sync::Arc;

use crate::features::plugins::{PluginManifest, PluginRegistry};

pub struct ListPluginsManifestsUseCase {
    plugin_registry: Arc<PluginRegistry>,
}

impl ListPluginsManifestsUseCase {
    pub fn new(plugin_registry: Arc<PluginRegistry>) -> Self {
        Self { plugin_registry }
    }

    pub async fn execute(&self) -> crate::Result<Vec<PluginManifest>> {
        self.plugin_registry.list_manifests()
    }
}
