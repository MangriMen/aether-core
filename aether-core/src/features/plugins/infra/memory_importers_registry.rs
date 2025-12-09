use async_trait::async_trait;
use dashmap::DashMap;

use crate::features::plugins::{CapabilityEntry, CapabilityRegistry, PluginError};

type CapabilityKey = (String, String); // (plugin_id, capability.id)

pub struct MemoryCapabilityRegistry<C: Send + Sync + Clone> {
    capabilities: DashMap<CapabilityKey, CapabilityEntry<C>>,
}

impl<C: Send + Sync + Clone> Default for MemoryCapabilityRegistry<C> {
    fn default() -> Self {
        Self {
            capabilities: DashMap::new(),
        }
    }
}

#[async_trait]
impl<C: Send + Sync + Clone> CapabilityRegistry<C> for MemoryCapabilityRegistry<C> {
    async fn list(&self) -> Result<Vec<CapabilityEntry<C>>, PluginError> {
        Ok(self
            .capabilities
            .iter()
            .map(|item| item.value().clone())
            .collect())
    }

    async fn add(
        &self,
        plugin_id: String,
        capability_id: String,
        capability: C,
    ) -> Result<(), PluginError> {
        let key = (plugin_id.to_string(), capability_id.to_string());
        self.capabilities.insert(
            key,
            CapabilityEntry {
                plugin_id: plugin_id.to_string(),
                capability,
            },
        );
        Ok(())
    }

    async fn find_by_capability_id(
        &self,
        capability_id: &str,
    ) -> Result<Vec<CapabilityEntry<C>>, PluginError> {
        let importers: Vec<CapabilityEntry<C>> = self
            .capabilities
            .iter()
            .filter(|item| item.key().1 == capability_id)
            .map(|item| item.value().clone())
            .collect();

        if importers.is_empty() {
            return Err(PluginError::ImporterNotFound {
                importer_id: capability_id.to_owned(),
            });
        }

        Ok(importers)
    }

    async fn find_by_plugin_and_capability_id(
        &self,
        plugin_id: String,
        capability_id: String,
    ) -> Result<CapabilityEntry<C>, PluginError> {
        let key = (plugin_id, capability_id.clone());
        let importer = self
            .capabilities
            .get(&key)
            .ok_or(PluginError::ImporterNotFound {
                importer_id: capability_id,
            })?;

        Ok(importer.clone())
    }

    async fn remove_by_plugin_and_capability(
        &self,
        plugin_id: String,
        capability_id: String,
    ) -> Result<(), PluginError> {
        let key = (plugin_id, capability_id);
        self.capabilities.remove(&key);
        Ok(())
    }
}
