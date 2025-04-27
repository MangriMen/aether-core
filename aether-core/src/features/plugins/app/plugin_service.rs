use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::features::plugins::{
    LoadConfigType, Plugin, PluginLoader, PluginSettingsStorage, PluginStorage,
};
use crate::{Error, ErrorKind, Result};

#[derive(Debug)]
pub struct PluginService<PS, PSS, PL>
where
    PS: PluginStorage + Send + Sync,
    PSS: PluginSettingsStorage + Send + Sync,
    PL: PluginLoader + Send + Sync,
{
    plugin_storage: PS,
    plugin_settings_storage: PSS,
    loaders: HashMap<LoadConfigType, PL>,
    plugins: HashMap<String, Plugin>,
}

impl<PS, PSS, PL> PluginService<PS, PSS, PL>
where
    PS: PluginStorage + Send + Sync,
    PSS: PluginSettingsStorage + Send + Sync,
    PL: PluginLoader + Send + Sync,
{
    pub fn new(
        plugin_storage: PS,
        plugin_settings_storage: PSS,
        loaders: HashMap<LoadConfigType, PL>,
    ) -> Self {
        Self {
            plugin_storage,
            plugin_settings_storage,
            loaders,
            plugins: HashMap::new(),
        }
    }

    pub async fn scan_plugins(&mut self) -> Result<()> {
        let found_plugins = self.plugin_storage.list().await?;
        self.sync_plugins(found_plugins).await
    }

    pub fn list(&self) -> impl Iterator<Item = &Plugin> {
        self.plugins.values()
    }

    pub fn get(&self, plugin: &str) -> Result<&Plugin> {
        self.plugins.get(plugin).ok_or_else(|| {
            Error::from(ErrorKind::PluginNotFoundError(format!(
                "Plugin {} not found",
                plugin
            )))
        })
    }

    pub fn get_mut(&mut self, plugin: &str) -> Result<&mut Plugin> {
        self.plugins.get_mut(plugin).ok_or_else(|| {
            Error::from(ErrorKind::PluginNotFoundError(format!(
                "Plugin {} not found",
                plugin
            )))
        })
    }

    pub async fn load_plugin(&mut self, plugin_id: &str) -> Result<()> {
        let plugin = self.get(plugin_id)?;

        let plugin_settings = self.plugin_settings_storage.get(plugin_id).await?;

        let loader = self.loaders.get(&(&plugin.manifest.load).into());
        if let Some(loader) = loader {
            let plugin_instance = loader.load(plugin, &plugin_settings).await?;

            let plugin = self.get_mut(plugin_id)?;
            plugin.instance = Some(Arc::new(Mutex::new(plugin_instance)));
        } else {
            return Err(ErrorKind::PluginLoadError(format!(
                "Not found loader for {:?}",
                &plugin.manifest.load
            ))
            .as_error());
        }

        Ok(())
    }

    pub async fn unload_plugin(&mut self, plugin_id: &str) -> Result<()> {
        let plugin = self.get(plugin_id)?;

        let loader = self.loaders.get(&(&plugin.manifest.load).into());

        if let Some(loader) = loader {
            if let Some(plugin_instance) = plugin.instance.clone() {
                loader.unload(plugin_instance.clone()).await?;

                let plugin = self.get_mut(plugin_id)?;
                plugin.instance = None;
            } else {
                return Err(ErrorKind::PluginLoadError(format!(
                    "Plugin {} is not loaded",
                    plugin_id
                ))
                .as_error());
            }
        } else {
            return Err(ErrorKind::PluginLoadError(format!(
                "Not found loader for {:?}",
                &plugin.manifest.load
            ))
            .as_error());
        }

        Ok(())
    }

    async fn remove_plugin(&mut self, plugin_id: &str) -> Result<()> {
        self.unload_plugin(plugin_id).await?;
        self.plugins.remove(plugin_id);
        Ok(())
    }

    async fn sync_plugins(&mut self, found_plugins: HashMap<String, Plugin>) -> Result<()> {
        let existing_plugins = self.plugins.keys().cloned().collect::<HashSet<_>>();
        let new_plugins = found_plugins.keys().cloned().collect::<HashSet<_>>();

        let changed_plugins =
            self.find_changed_plugins(&existing_plugins, &new_plugins, &found_plugins);
        let plugins_to_add =
            self.determine_plugins_to_add(&existing_plugins, &new_plugins, &changed_plugins);

        self.remove_non_existing_plugins(&existing_plugins, &new_plugins)
            .await?;
        self.remove_changed_plugins(&changed_plugins).await?;
        self.add_new_plugins(&plugins_to_add, &found_plugins)
            .await?;

        Ok(())
    }

    fn find_changed_plugins(
        &self,
        existing_plugins: &HashSet<String>,
        new_plugins: &HashSet<String>,
        found_plugins: &HashMap<String, Plugin>,
    ) -> HashSet<String> {
        existing_plugins
            .intersection(new_plugins)
            .filter(|&plugin| {
                let old = self.plugins.get(plugin);
                let new = found_plugins.get(plugin);

                match (old, new) {
                    (Some(old), Some(new)) => old.manifest != new.manifest || old.hash != new.hash,
                    _ => false,
                }
            })
            .cloned()
            .collect()
    }

    fn determine_plugins_to_add(
        &self,
        existing_plugins: &HashSet<String>,
        new_plugins: &HashSet<String>,
        changed_plugins: &HashSet<String>,
    ) -> HashSet<String> {
        new_plugins
            .difference(existing_plugins)
            .chain(changed_plugins.iter())
            .cloned()
            .collect()
    }

    async fn remove_non_existing_plugins(
        &mut self,
        existing_plugins: &HashSet<String>,
        new_plugins: &HashSet<String>,
    ) -> Result<()> {
        for plugin in existing_plugins.difference(new_plugins) {
            log::debug!("Removing plugin {}", plugin);
            self.remove_plugin(plugin).await?;
        }
        Ok(())
    }

    async fn remove_changed_plugins(&mut self, changed_plugins: &HashSet<String>) -> Result<()> {
        for plugin in changed_plugins {
            log::debug!("Removing changed plugin {}", plugin);
            self.remove_plugin(plugin).await?;
        }
        Ok(())
    }

    async fn add_new_plugins(
        &mut self,
        plugins_to_add: &HashSet<String>,
        found_plugins: &HashMap<String, Plugin>,
    ) -> Result<()> {
        for plugin in plugins_to_add {
            if let Some(plugin_state) = found_plugins.get(plugin) {
                log::debug!("Adding plugin {}", plugin);
                self.plugins.insert(plugin.clone(), plugin_state.clone());
            }
        }
        Ok(())
    }
}
