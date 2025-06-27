use std::collections::HashSet;

use dashmap::DashMap;

use crate::features::plugins::{Plugin, PluginError, PluginManifest};

use dashmap::mapref::{
    multiple::RefMulti as DashMapRefMulti,
    one::{Ref as DashMapRef, RefMut as DashMapRefMut},
};

#[derive(Default)]
pub struct PluginRegistry {
    plugins: DashMap<String, Plugin>,
}

impl PluginRegistry {
    pub fn insert(&self, plugin_id: String, plugin: Plugin) {
        self.plugins.insert(plugin_id, plugin);
    }

    pub fn list(&self) -> impl Iterator<Item = DashMapRefMulti<'_, String, Plugin>> {
        self.plugins.iter()
    }

    pub fn get(&self, plugin_id: &str) -> Result<DashMapRef<'_, String, Plugin>, PluginError> {
        self.plugins
            .get(plugin_id)
            .ok_or_else(|| PluginError::PluginNotFoundError {
                plugin_id: plugin_id.to_owned(),
            })
    }

    pub fn get_mut(
        &self,
        plugin_id: &str,
    ) -> Result<DashMapRefMut<'_, String, Plugin>, PluginError> {
        self.plugins
            .get_mut(plugin_id)
            .ok_or_else(|| PluginError::PluginNotFoundError {
                plugin_id: plugin_id.to_owned(),
            })
    }

    pub fn remove(&self, plugin_id: &str) {
        self.plugins.remove(plugin_id);
    }

    pub fn get_ids(&self) -> HashSet<String> {
        self.plugins
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    pub fn list_manifests(&self) -> Result<Vec<PluginManifest>, PluginError> {
        Ok(self.list().map(|plugin| plugin.manifest.clone()).collect())
    }

    pub fn get_manifest(&self, plugin_id: &str) -> Result<PluginManifest, PluginError> {
        Ok(self.get(plugin_id)?.manifest.clone())
    }
}
