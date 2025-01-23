use std::collections::HashMap;

use super::{InstancePlugin, InstancePluginMetadata, PackwizPlugin};

#[derive(Debug)]
pub struct PluginState {
    pub metadata: InstancePluginMetadata,
    pub plugin: Box<dyn InstancePlugin>,
}

impl PluginState {
    pub fn new(plugin: Box<dyn InstancePlugin>) -> Self {
        Self {
            metadata: InstancePluginMetadata { is_loaded: false },
            plugin,
        }
    }

    async fn enable(&mut self) -> crate::Result<()> {
        self.plugin.init().await?;
        self.metadata.is_loaded = true;

        Ok(())
    }

    async fn disable(&mut self) -> crate::Result<()> {
        self.plugin.unload().await?;
        self.metadata.is_loaded = false;

        Ok(())
    }
}

#[derive(Debug)]
pub struct PluginManager {
    pub plugins: HashMap<String, PluginState>,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    fn plugin_state_from_plugin(plugin: Box<dyn InstancePlugin>) -> (String, PluginState) {
        (plugin.get_id().to_string(), PluginState::new(plugin))
    }

    pub fn scan_plugins(
        &mut self,
        //  _path: &PathBuf
    ) {
        let found_plugins: Vec<Box<dyn InstancePlugin>> = vec![Box::new(PackwizPlugin {
            id: "packwiz".to_string(),
            name: "Packwiz".to_string(),
            description: "A plugin for managing packs".to_string(),
        })];

        let plugins: HashMap<String, PluginState> = found_plugins
            .into_iter()
            .map(PluginManager::plugin_state_from_plugin)
            .collect();

        self.plugins = plugins;
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

    pub async fn enable_plugin(&mut self, plugin: String) -> crate::Result<()> {
        let plugin = self.get_plugin_mut(&plugin)?;
        plugin.enable().await?;

        Ok(())
    }

    pub async fn disable_plugin(&mut self, plugin: String) -> crate::Result<()> {
        let plugin = self.get_plugin_mut(&plugin)?;
        plugin.disable().await?;

        Ok(())
    }
}
