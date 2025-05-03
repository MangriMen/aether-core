use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::plugins::{PluginManifest, PluginRegistry},
    shared::domain::AsyncUseCaseWithError,
};

pub struct ListPluginsManifestsUseCase {
    plugin_registry: Arc<PluginRegistry>,
}

impl ListPluginsManifestsUseCase {
    pub fn new(plugin_registry: Arc<PluginRegistry>) -> Self {
        Self { plugin_registry }
    }
}

#[async_trait]
impl AsyncUseCaseWithError for ListPluginsManifestsUseCase {
    type Output = Vec<PluginManifest>;
    type Error = crate::Error;

    async fn execute(&self) -> Result<Self::Output, Self::Error> {
        self.plugin_registry.list_manifests()
    }
}
