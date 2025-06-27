use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::features::{
    plugins::{Plugin, PluginError, PluginLoader, PluginRegistry, PluginStorage},
    settings::SettingsStorage,
};

use super::DisablePluginUseCase;

pub struct SyncPluginsUseCase<PS: PluginStorage, SS: SettingsStorage, PL: PluginLoader> {
    plugin_storage: Arc<PS>,
    plugin_registry: Arc<PluginRegistry>,
    disable_plugin_use_case: DisablePluginUseCase<SS, PL>,
}

impl<PS: PluginStorage, SS: SettingsStorage, PL: PluginLoader> SyncPluginsUseCase<PS, SS, PL> {
    pub fn new(
        plugin_storage: Arc<PS>,
        plugin_registry: Arc<PluginRegistry>,
        disable_plugin_use_case: DisablePluginUseCase<SS, PL>,
    ) -> Self {
        Self {
            plugin_storage,
            plugin_registry,
            disable_plugin_use_case,
        }
    }

    async fn sync_plugins(
        &self,
        found_plugins: HashMap<String, Plugin>,
    ) -> Result<(), PluginError> {
        let existing_plugins = self.plugin_registry.get_ids();

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
                let old = self.plugin_registry.get(plugin);
                let new = found_plugins.get(plugin);

                match (old, new) {
                    (Ok(old), Some(new)) => old.manifest != new.manifest || old.hash != new.hash,
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
        &self,
        existing_plugins: &HashSet<String>,
        new_plugins: &HashSet<String>,
    ) -> Result<(), PluginError> {
        for plugin_id in existing_plugins.difference(new_plugins) {
            log::debug!("Removing plugin {}", plugin_id);
            self.remove_plugin(plugin_id).await?;
        }
        Ok(())
    }

    async fn remove_changed_plugins(
        &self,
        changed_plugins: &HashSet<String>,
    ) -> Result<(), PluginError> {
        for plugin_id in changed_plugins {
            log::debug!("Removing changed plugin {}", plugin_id);
            self.remove_plugin(plugin_id).await?;
        }
        Ok(())
    }

    async fn add_new_plugins(
        &self,
        plugins_to_add: &HashSet<String>,
        found_plugins: &HashMap<String, Plugin>,
    ) -> Result<(), PluginError> {
        for plugin_id in plugins_to_add {
            if let Some(plugin_state) = found_plugins.get(plugin_id) {
                log::debug!("Adding plugin {}", plugin_id);
                self.plugin_registry
                    .insert(plugin_id.clone(), plugin_state.clone());
            }
        }
        Ok(())
    }

    async fn remove_plugin(&self, plugin_id: &str) -> Result<(), PluginError> {
        self.disable_plugin_use_case
            .execute(plugin_id.to_string())
            .await?;
        self.plugin_registry.remove(plugin_id);
        Ok(())
    }

    pub async fn execute(&self) -> Result<(), PluginError> {
        let found_plugins = self.plugin_storage.list().await?;
        self.sync_plugins(found_plugins).await
    }
}
