use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use super::PluginState;

#[derive(Debug, Default)]
pub struct PluginManager {
    plugins: HashMap<String, PluginState>,
}

impl PluginManager {
    pub async fn get_plugins_from_dir(
        &mut self,
        path: &Path,
    ) -> crate::Result<HashMap<String, PluginState>> {
        let mut stream = tokio::fs::read_dir(path).await?;
        let mut found_plugins = HashMap::new();

        while let Some(dir) = stream.next_entry().await? {
            match PluginState::from_dir(&dir.path()).await {
                Ok(plugin_state) => {
                    found_plugins.insert(plugin_state.metadata.plugin.id.clone(), plugin_state);
                }
                Err(e) => {
                    log::debug!("Failed to load plugin: {:?}. {e}", dir);
                }
            };
        }

        Ok(found_plugins)
    }

    pub async fn scan_plugins(&mut self, path: &Path) -> crate::Result<()> {
        let found_plugins: HashMap<_, _> = self.get_plugins_from_dir(path).await?;
        let existing_plugins: HashSet<_> = self.plugins.keys().cloned().collect();
        let new_plugins: HashSet<_> = found_plugins.keys().cloned().collect();

        let mut changed_plugins: HashSet<String> = HashSet::new();
        for plugin in &existing_plugins & &new_plugins {
            if let (Some(old), Some(new)) = (self.plugins.get(&plugin), found_plugins.get(&plugin))
            {
                if old.metadata != new.metadata || old.plugin_hash != new.plugin_hash {
                    changed_plugins.insert(plugin.clone());
                }
            }
        }

        let plugins_to_add: HashSet<String> = (&new_plugins - &existing_plugins)
            .union(&changed_plugins)
            .cloned()
            .collect();

        // Removing old plugins
        for plugin in &existing_plugins - &new_plugins {
            log::debug!("Removing plugin {plugin}");
            self.remove_plugin(&plugin).await?;
        }

        // Removing changed plugins
        for plugin in &changed_plugins {
            log::debug!("Removing change plugin {plugin}");
            self.remove_plugin(plugin).await?;
        }

        // Adding new or changed plugins
        for plugin in &plugins_to_add {
            if let Some(plugin_state) = found_plugins.get(plugin) {
                log::debug!("Adding plugin {plugin}");
                self.plugins.insert(plugin.clone(), plugin_state.clone());
            }
        }

        Ok(())
    }

    pub fn get_plugins(&self) -> impl Iterator<Item = &PluginState> {
        self.plugins.values()
    }

    pub fn get_plugin(&self, plugin: &str) -> crate::Result<&PluginState> {
        self.plugins.get(plugin).ok_or_else(|| {
            crate::ErrorKind::PluginNotFoundError(format!("Plugin {plugin} not found")).as_error()
        })
    }

    pub fn get_plugin_mut(&mut self, plugin: &str) -> crate::Result<&mut PluginState> {
        self.plugins.get_mut(plugin).ok_or_else(|| {
            crate::ErrorKind::PluginNotFoundError(format!("Plugin {plugin} not found")).as_error()
        })
    }

    pub async fn load_plugin(&mut self, plugin: &str) -> crate::Result<()> {
        self.get_plugin_mut(plugin)?.load().await
    }

    pub async fn unload_plugin(&mut self, plugin: &str) -> crate::Result<()> {
        self.get_plugin_mut(plugin)?.unload().await
    }

    async fn remove_plugin(&mut self, plugin: &str) -> crate::Result<()> {
        self.get_plugin_mut(plugin)?.unload().await?;
        self.plugins.remove(plugin);
        Ok(())
    }
}
