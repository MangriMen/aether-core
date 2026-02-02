use async_trait::async_trait;

use crate::features::plugins::{CapabilityEntry, PluginError};

#[async_trait]
pub trait CapabilityRegistry<C: Send + Sync + Clone>: Send + Sync {
    async fn add(
        &self,
        plugin_id: String,
        capability_id: String,
        capability: C,
    ) -> Result<(), PluginError>;

    async fn list(&self) -> Result<Vec<CapabilityEntry<C>>, PluginError>;

    async fn find_by_capability_id(
        &self,
        capability_id: &str,
    ) -> Result<Vec<CapabilityEntry<C>>, PluginError>;

    async fn find_by_plugin_and_capability_id(
        &self,
        plugin_id: &str,
        capability_id: &str,
    ) -> Result<CapabilityEntry<C>, PluginError>;

    async fn remove_by_plugin_and_capability(
        &self,
        plugin_id: String,
        capability_id: String,
    ) -> Result<(), PluginError>;
}
