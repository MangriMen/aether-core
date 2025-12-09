use std::sync::Arc;

use crate::features::plugins::{
    CapabilityEntry, CapabilityRegistry, ImporterCapability, PluginError,
};

pub struct ListImportersUseCase<IR: CapabilityRegistry<ImporterCapability>> {
    importers_registry: Arc<IR>,
}

impl<IR: CapabilityRegistry<ImporterCapability>> ListImportersUseCase<IR> {
    pub fn new(importers_registry: Arc<IR>) -> Self {
        Self { importers_registry }
    }

    pub async fn execute(&self) -> Result<Vec<CapabilityEntry<ImporterCapability>>, PluginError> {
        self.importers_registry.list().await
    }
}
