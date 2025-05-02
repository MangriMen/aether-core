use crate::{
    core::domain::{LazyLocator, ServiceLocator},
    features::plugins::{
        DisablePluginUseCase, EditPluginSettings, EditPluginSettingsUseCase, EnablePluginUseCase,
        GetPluginManifestUseCase, GetPluginSettingsUseCase, ListPluginsManifestsUseCase,
        PluginManifest, PluginSettings,
    },
    shared::domain::{AsyncUseCaseWithError, AsyncUseCaseWithInputAndError},
};

#[tracing::instrument]
pub async fn scan() -> crate::Result<()> {
    let service_locator = ServiceLocator::get().await?;
    let mut plugin_service = service_locator.plugin_service.write().await;

    plugin_service.scan_plugins().await
}

#[tracing::instrument]
pub async fn list_manifests() -> crate::Result<Vec<PluginManifest>> {
    let lazy_locator = LazyLocator::get().await?;

    ListPluginsManifestsUseCase::new(lazy_locator.get_plugin_registry().await)
        .execute()
        .await
}

#[tracing::instrument]
pub async fn get_manifest(plugin_id: String) -> crate::Result<PluginManifest> {
    let lazy_locator = LazyLocator::get().await?;

    GetPluginManifestUseCase::new(lazy_locator.get_plugin_registry().await)
        .execute(plugin_id)
        .await
}

#[tracing::instrument]
pub async fn is_enabled(id: &str) -> crate::Result<bool> {
    let service_locator = ServiceLocator::get().await?;
    let plugin_service = service_locator.plugin_service.read().await;

    Ok(plugin_service.get(id)?.is_loaded())
}

#[tracing::instrument]
pub async fn enable(plugin_id: String) -> crate::Result<()> {
    let lazy_locator = LazyLocator::get().await?;

    EnablePluginUseCase::new(
        lazy_locator.get_plugin_registry().await,
        lazy_locator.get_plugin_loader_registry().await,
        lazy_locator.get_plugin_settings_storage().await,
        lazy_locator.get_settings_storage().await,
    )
    .execute(plugin_id)
    .await
}

#[tracing::instrument]
pub async fn disable(plugin_id: String) -> crate::Result<()> {
    let lazy_locator = LazyLocator::get().await?;

    DisablePluginUseCase::new(
        lazy_locator.get_plugin_registry().await,
        lazy_locator.get_plugin_loader_registry().await,
        lazy_locator.get_settings_storage().await,
    )
    .execute(plugin_id)
    .await
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
pub async fn get_settings(id: String) -> crate::Result<Option<PluginSettings>> {
    let lazy_locator = LazyLocator::get().await?;

    GetPluginSettingsUseCase::new(lazy_locator.get_plugin_settings_storage().await)
        .execute(id)
        .await
}

#[tracing::instrument]
pub async fn edit_settings(id: String, edit_settings: EditPluginSettings) -> crate::Result<()> {
    let lazy_locator = LazyLocator::get().await?;

    EditPluginSettingsUseCase::new(lazy_locator.get_plugin_settings_storage().await)
        .execute((id, edit_settings))
        .await
}
