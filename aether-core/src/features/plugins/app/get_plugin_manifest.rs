use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::plugins::{PluginManifest, PluginRegistry},
    shared::domain::AsyncUseCaseWithInputAndError,
};

pub struct GetPluginManifestUseCase {
    plugin_registry: Arc<PluginRegistry>,
}

impl GetPluginManifestUseCase {
    pub fn new(plugin_registry: Arc<PluginRegistry>) -> Self {
        Self { plugin_registry }
    }
}

#[async_trait]
impl AsyncUseCaseWithInputAndError for GetPluginManifestUseCase {
    type Input = String;
    type Output = PluginManifest;
    type Error = crate::Error;

    async fn execute(&self, plugin_id: Self::Input) -> Result<Self::Output, Self::Error> {
        self.plugin_registry.get_manifest(&plugin_id)
    }
}
