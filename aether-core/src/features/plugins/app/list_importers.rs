use std::sync::Arc;

use crate::features::{
    instance::{Importer, ImporterCapability},
    plugins::{CapabilityEntry, CapabilityRegistry, PluginError},
};

pub struct ListImportersUseCase<IR: CapabilityRegistry<Arc<dyn Importer>>> {
    importers_registry: Arc<IR>,
}

impl<IR: CapabilityRegistry<Arc<dyn Importer>>> ListImportersUseCase<IR> {
    pub fn new(importers_registry: Arc<IR>) -> Self {
        Self { importers_registry }
    }

    pub async fn execute(&self) -> Result<Vec<CapabilityEntry<ImporterCapability>>, PluginError> {
        Ok(self
            .importers_registry
            .list()
            .await?
            .into_iter()
            .map(|entry| CapabilityEntry {
                plugin_id: entry.plugin_id,
                capability: entry.capability.info().clone(),
            })
            .collect())
    }
}
