use async_trait::async_trait;
use dashmap::DashMap;

use crate::features::plugins::{Importer, ImportersRegistry, PluginError};

#[derive(Default)]
pub struct MemoryImportersRegistry {
    importers: DashMap<String, Importer>,
}

#[async_trait]
impl ImportersRegistry for MemoryImportersRegistry {
    async fn list(&self) -> Result<Vec<Importer>, PluginError> {
        Ok(self
            .importers
            .iter()
            .map(|item| item.value().clone())
            .collect())
    }

    async fn add(&self, importer: Importer) -> Result<(), PluginError> {
        self.importers
            .insert(importer.capability.id.clone(), importer);
        Ok(())
    }

    async fn get(&self, importer_id: &str) -> Result<Importer, PluginError> {
        let importer =
            self.importers
                .get(importer_id)
                .ok_or_else(|| PluginError::ImporterNotFound {
                    importer_id: importer_id.to_owned(),
                })?;

        Ok(importer.clone())
    }

    async fn remove(&self, importer_id: &str) -> Result<(), PluginError> {
        self.importers.remove(importer_id);
        Ok(())
    }
}
