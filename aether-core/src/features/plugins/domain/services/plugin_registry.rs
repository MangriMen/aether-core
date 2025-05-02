use dashmap::DashMap;

use crate::{
    features::plugins::{Plugin, PluginManifest},
    ErrorKind,
};

use dashmap::mapref::{
    multiple::RefMulti as DashMapRefMulti,
    one::{Ref as DashMapRef, RefMut as DashMapRefMut},
};

#[derive(Debug, Default)]
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

    pub fn get(&self, plugin_id: &str) -> crate::Result<DashMapRef<'_, String, Plugin>> {
        self.plugins.get(plugin_id).ok_or_else(|| {
            ErrorKind::PluginNotFoundError(format!("Plugin {} not found", plugin_id)).as_error()
        })
    }

    pub fn get_mut(&self, plugin_id: &str) -> crate::Result<DashMapRefMut<'_, String, Plugin>> {
        self.plugins.get_mut(plugin_id).ok_or_else(|| {
            ErrorKind::PluginNotFoundError(format!("Plugin {} not found", plugin_id)).as_error()
        })
    }

    pub fn remove(&self, plugin_id: &str) {
        self.plugins.remove(plugin_id);
    }

    pub fn list_manifests(&self) -> crate::Result<Vec<PluginManifest>> {
        Ok(self.list().map(|plugin| plugin.manifest.clone()).collect())
    }

    pub fn get_manifest(&self, plugin_id: &str) -> crate::Result<PluginManifest> {
        Ok(self.get(plugin_id)?.manifest.clone())
    }
}
