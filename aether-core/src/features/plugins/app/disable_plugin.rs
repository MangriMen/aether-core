use std::sync::Arc;

use crate::features::{
    plugins::{LoadConfigType, PluginError, PluginLoader, PluginLoaderRegistry, PluginRegistry},
    settings::SettingsStorage,
};

pub struct DisablePluginUseCase<SS, PL> {
    plugin_registry: Arc<PluginRegistry>,
    plugin_loader_registry: Arc<PluginLoaderRegistry<PL>>,
    settings_storage: Arc<SS>,
}

impl<SS: SettingsStorage, PL: PluginLoader> DisablePluginUseCase<SS, PL> {
    pub fn new(
        plugin_registry: Arc<PluginRegistry>,
        plugin_loader_registry: Arc<PluginLoaderRegistry<PL>>,
        settings_storage: Arc<SS>,
    ) -> Self {
        Self {
            plugin_registry,
            plugin_loader_registry,
            settings_storage,
        }
    }

    pub async fn execute(&self, plugin_id: String) -> Result<(), PluginError> {
        let plugin = self.plugin_registry.get(&plugin_id)?;
        let manifest = plugin.manifest.clone();
        let instance = plugin.instance.clone();
        drop(plugin);

        let load_config_type: LoadConfigType = (&(manifest.load)).into();
        let loader = self.plugin_loader_registry.get(&load_config_type)?;

        if let Some(plugin_instance) = instance {
            loader.unload(plugin_instance.clone()).await?;

            let mut plugin = self.plugin_registry.get_mut(&plugin_id)?;
            plugin.instance = None;
        } else {
            return Err(PluginError::PluginNotFoundError { plugin_id });
        }

        let mut settings = self.settings_storage.get().await?;
        if !settings.enabled_plugins.contains(&plugin_id) {
            settings.enabled_plugins.remove(&plugin_id);
            self.settings_storage.upsert(settings).await?;
        }

        Ok(())
    }
}
