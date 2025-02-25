use std::{collections::HashMap, path::Path};

use super::PluginState;

#[derive(Debug, Default)]
pub struct PluginManager {
    plugins: HashMap<String, PluginState>,
}

impl PluginManager {
    pub async fn scan_plugins(&mut self, path: &Path) -> crate::Result<()> {
        let mut stream = tokio::fs::read_dir(path).await?;

        while let Some(dir) = &stream.next_entry().await? {
            match PluginState::from_dir(&dir.path()) {
                Ok(plugin_state) => {
                    self.plugins
                        .insert(plugin_state.metadata.plugin.id.clone(), plugin_state);
                }
                Err(e) => {
                    log::debug!("Failed to load plugin: {}\n{e}", dir.path().display());
                }
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

    fn get_plugin_mut(&mut self, plugin: &str) -> crate::Result<&mut PluginState> {
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
}
