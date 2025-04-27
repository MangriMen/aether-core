use crate::{
    core::domain::ServiceLocator,
    features::plugins::{
        EditPluginSettings, PluginManifest, PluginSettings, PluginSettingsManager,
    },
};

#[tracing::instrument]
pub async fn scan() -> crate::Result<()> {
    let service_locator = ServiceLocator::get().await?;
    let mut plugin_service = service_locator.plugin_service.write().await;

    plugin_service.scan_plugins().await
}

#[tracing::instrument]
pub async fn list() -> crate::Result<Vec<PluginManifest>> {
    let service_locator = ServiceLocator::get().await?;
    let plugin_service = service_locator.plugin_service.read().await;

    Ok(plugin_service.list_manifests().cloned().collect())
}

#[tracing::instrument]
pub async fn get(id: &str) -> crate::Result<PluginManifest> {
    let service_locator = ServiceLocator::get().await?;
    let plugin_service = service_locator.plugin_service.read().await;

    plugin_service.get_manifest(id)
}

#[tracing::instrument]
pub async fn is_enabled(id: &str) -> crate::Result<bool> {
    let service_locator = ServiceLocator::get().await?;
    let plugin_service = service_locator.plugin_service.read().await;

    Ok(plugin_service.get(id)?.is_loaded())
}

#[tracing::instrument]
pub async fn enable(id: &str) -> crate::Result<()> {
    let service_locator = ServiceLocator::get().await?;
    let mut plugin_service = service_locator.plugin_service.write().await;

    plugin_service.enable(id).await
}

#[tracing::instrument]
pub async fn disable(id: &str) -> crate::Result<()> {
    let service_locator = ServiceLocator::get().await?;
    let mut plugin_service = service_locator.plugin_service.write().await;

    plugin_service.disable(id).await
}

#[tracing::instrument]
pub async fn call(id: &str, data: &str) -> crate::Result<()> {
    let service_locator = ServiceLocator::get().await?;

    let plugin_service = service_locator.plugin_service.read().await;

    let _plugin = plugin_service.get(id)?;

    log::debug!("Calling plugin {:?}", id);
    // plugin.plugin.call(data).await?;

    Ok(())
}

#[tracing::instrument]
pub async fn get_settings(id: &str) -> crate::Result<Option<PluginSettings>> {
    let service_locator = ServiceLocator::get().await?;
    service_locator.plugin_settings_manager.get(id).await
}

#[tracing::instrument]
pub async fn edit_settings(id: &str, edit_settings: &EditPluginSettings) -> crate::Result<()> {
    let service_locator = ServiceLocator::get().await?;
    service_locator
        .plugin_settings_manager
        .edit(id, edit_settings)
        .await
}
