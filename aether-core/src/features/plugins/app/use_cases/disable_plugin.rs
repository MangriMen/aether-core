use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::{
        plugins::{PluginLoader, PluginLoaderRegistry, PluginRegistry},
        settings::SettingsStorage,
    },
    shared::domain::AsyncUseCaseWithInputAndError,
    ErrorKind,
};

pub struct DisablePluginUseCase<SS, PL> {
    plugin_registry: Arc<PluginRegistry>,
    plugin_loader_registry: Arc<PluginLoaderRegistry<PL>>,
    settings_storage: Arc<SS>,
}

impl<SS, PL> DisablePluginUseCase<SS, PL> {
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
}

#[async_trait]
impl<SS, PL> AsyncUseCaseWithInputAndError for DisablePluginUseCase<SS, PL>
where
    SS: SettingsStorage + Send + Sync,
    PL: PluginLoader + Send + Sync,
{
    type Input = String;
    type Output = ();
    type Error = crate::Error;

    async fn execute(&self, plugin_id: Self::Input) -> Result<Self::Output, Self::Error> {
        let plugin = self.plugin_registry.get(&plugin_id)?;

        let loader = self
            .plugin_loader_registry
            .get(&(&plugin.manifest.load).into())?;

        if let Some(plugin_instance) = plugin.instance.clone() {
            loader.unload(plugin_instance.clone()).await?;

            let mut plugin = self.plugin_registry.get_mut(&plugin_id)?;
            plugin.instance = None;
        } else {
            return Err(
                ErrorKind::PluginLoadError(format!("Plugin {} is not loaded", plugin_id))
                    .as_error(),
            );
        }

        let mut settings = self.settings_storage.get().await?;
        if !settings.enabled_plugins.contains(&plugin_id) {
            settings.enabled_plugins.remove(&plugin_id);
            self.settings_storage.upsert(&settings).await?;
        }

        Ok(())
    }
}
