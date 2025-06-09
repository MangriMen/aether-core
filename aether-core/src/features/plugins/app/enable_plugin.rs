use std::sync::Arc;

use tokio::sync::Mutex;

use crate::features::{
    plugins::{PluginLoader, PluginLoaderRegistry, PluginRegistry, PluginSettingsStorage},
    settings::SettingsStorage,
};

pub struct EnablePluginUseCase<PSS: PluginSettingsStorage, SS: SettingsStorage, PL: PluginLoader> {
    plugin_registry: Arc<PluginRegistry>,
    plugin_loader_registry: Arc<PluginLoaderRegistry<PL>>,
    plugin_settings_storage: Arc<PSS>,
    settings_storage: Arc<SS>,
}

impl<PSS: PluginSettingsStorage, SS: SettingsStorage, PL: PluginLoader>
    EnablePluginUseCase<PSS, SS, PL>
{
    pub fn new(
        plugin_registry: Arc<PluginRegistry>,
        plugin_loader_registry: Arc<PluginLoaderRegistry<PL>>,
        plugin_settings_storage: Arc<PSS>,
        settings_storage: Arc<SS>,
    ) -> Self {
        Self {
            plugin_registry,
            plugin_loader_registry,
            plugin_settings_storage,
            settings_storage,
        }
    }

    pub async fn execute(&self, plugin_id: String) -> crate::Result<()> {
        let plugin = self.plugin_registry.get(&plugin_id)?;

        let plugin_settings = self.plugin_settings_storage.get(&plugin_id).await?;

        let loader = self
            .plugin_loader_registry
            .get(&(&plugin.manifest.load).into())?;

        let plugin_instance = loader.load(&plugin, &plugin_settings).await?;

        let mut plugin = self.plugin_registry.get_mut(&plugin_id)?;
        plugin.instance = Some(Arc::new(Mutex::new(plugin_instance)));

        let mut settings = self.settings_storage.get().await?;
        if !settings.enabled_plugins.contains(&plugin_id) {
            settings.enabled_plugins.insert(plugin_id.to_string());
            self.settings_storage.upsert(settings).await?;
        }

        Ok(())
    }
}
