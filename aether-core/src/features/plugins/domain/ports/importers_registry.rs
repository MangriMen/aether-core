use async_trait::async_trait;

use crate::features::plugins::{Importer, PluginError};

#[async_trait]
pub trait ImportersRegistry: Send + Sync {
    async fn add(&self, importer: Importer) -> Result<(), PluginError>;
    async fn list(&self) -> Result<Vec<Importer>, PluginError>;
    async fn get(&self, importer_id: &str) -> Result<Importer, PluginError>;
    async fn remove(&self, importer_id: &str) -> Result<(), PluginError>;
}
