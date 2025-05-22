use crate::{
    core::domain::LazyLocator,
    features::plugins::{
        DisablePluginUseCase, EditPluginSettings, EditPluginSettingsUseCase, EnablePluginUseCase,
        GetPluginManifestUseCase, GetPluginSettingsUseCase, ListPluginsManifestsUseCase,
        PluginManifest, PluginSettings, SyncPluginsUseCase,
    },
};

#[tracing::instrument]
pub async fn sync() -> crate::Result<()> {
    let lazy_locator = LazyLocator::get().await?;

    let disable_plugin_use_case = DisablePluginUseCase::new(
        lazy_locator.get_plugin_registry().await,
        lazy_locator.get_plugin_loader_registry().await,
        lazy_locator.get_settings_storage().await,
    );

    SyncPluginsUseCase::new(
        lazy_locator.get_plugin_storage().await,
        lazy_locator.get_plugin_registry().await,
        disable_plugin_use_case,
    )
    .execute()
    .await
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
    let lazy_locator = LazyLocator::get().await?;
    let plugin_registry = lazy_locator.get_plugin_registry().await;
    let plugin = plugin_registry.get(id)?;
    Ok(plugin.is_loaded())
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
pub async fn call(plugin_id: String, data: String) -> crate::Result<()> {
    let lazy_locator = LazyLocator::get().await?;

    let plugin_registry = lazy_locator.get_plugin_registry().await;
    let _plugin = plugin_registry.get(&plugin_id);

    log::debug!("Calling plugin {:?}", plugin_id);
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
        .execute(id, edit_settings)
        .await
}
