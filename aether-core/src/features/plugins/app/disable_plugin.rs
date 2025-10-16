use std::sync::Arc;

use tokio::sync::Mutex;

use crate::features::{
    plugins::{
        LoadConfigType, PluginError, PluginInstance, PluginLoader, PluginLoaderRegistry,
        PluginManifest, PluginRegistry, PluginState,
    },
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
        let mut plugin = self.plugin_registry.get_mut(&plugin_id)?;

        let plugin_instance = self.check_is_able_to_unload(&plugin.state)?;

        plugin.state = PluginState::Unloading;
        let manifest = plugin.manifest.clone();
        // Drop immediate to prevent dead lock in dash map
        drop(plugin);

        match self
            .unload_plugin(&plugin_id, &manifest, plugin_instance.clone())
            .await
        {
            Ok(_) => self.remove_from_enabled_plugins(&plugin_id).await,
            Err(err) => {
                let mut plugin = self.plugin_registry.get_mut(&plugin_id)?;
                match &err {
                    // If there is error on plugin on_unload call we are anyway drop instance
                    PluginError::CallError { .. } => plugin.state = PluginState::NotLoaded,
                    // If there is others error - plugin still loaded
                    _ => plugin.state = PluginState::Loaded(plugin_instance),
                }

                Err(err)
            }
        }
    }

    fn check_is_able_to_unload(
        &self,
        plugin_state: &PluginState,
    ) -> Result<Arc<Mutex<dyn PluginInstance>>, PluginError> {
        match plugin_state {
            PluginState::Loaded(plugin_instance) => Ok(plugin_instance.clone()),
            PluginState::Loading => Err(PluginError::PluginAlreadyLoading),
            PluginState::NotLoaded | PluginState::Unloading | PluginState::Failed(_) => {
                Err(PluginError::PluginAlreadyUnloaded)
            }
        }
    }

    async fn unload_plugin(
        &self,
        plugin_id: &str,
        manifest: &PluginManifest,
        plugin_instance: Arc<Mutex<dyn PluginInstance>>,
    ) -> Result<(), PluginError> {
        let load_config_type: LoadConfigType = (&manifest.load).into();
        let loader = self.plugin_loader_registry.get(&load_config_type)?;

        loader.unload(plugin_instance.clone()).await?;

        let mut plugin = self.plugin_registry.get_mut(plugin_id)?;
        plugin.state = PluginState::NotLoaded;

        Ok(())
    }

    async fn remove_from_enabled_plugins(&self, plugin_id: &str) -> Result<(), PluginError> {
        let mut settings = self.settings_storage.get().await?;

        if settings.enabled_plugins.contains(plugin_id) {
            settings.enabled_plugins.remove(plugin_id);
            self.settings_storage.upsert(settings).await?;
        }

        Ok(())
    }
}
