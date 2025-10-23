use std::sync::Arc;

use crate::features::plugins::{Importer, ImportersRegistry, PluginError};

pub struct ListImportersUseCase<IR: ImportersRegistry> {
    importers_registry: Arc<IR>,
}

impl<IR: ImportersRegistry> ListImportersUseCase<IR> {
    pub fn new(importers_registry: Arc<IR>) -> Self {
        Self { importers_registry }
    }

    pub async fn execute(&self) -> Result<Vec<Importer>, PluginError> {
        self.importers_registry.list().await
    }
}
