use crate::{
    core::domain::LazyLocator,
    features::plugins::{
        DisablePluginUseCase, EditPluginSettings, EditPluginSettingsUseCase, EnablePluginUseCase,
        GetPluginDtoUseCase, GetPluginSettingsUseCase, ListPluginsDtoUseCase, PluginDto,
        PluginSettings, SyncPluginsUseCase,
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

    Ok(SyncPluginsUseCase::new(
        lazy_locator.get_plugin_storage().await,
        lazy_locator.get_plugin_registry().await,
        disable_plugin_use_case,
    )
    .execute()
    .await?)
}

#[tracing::instrument]
pub async fn list() -> crate::Result<Vec<PluginDto>> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        ListPluginsDtoUseCase::new(lazy_locator.get_plugin_registry().await)
            .execute()
            .await?,
    )
}

#[tracing::instrument]
pub async fn get(plugin_id: String) -> crate::Result<PluginDto> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        GetPluginDtoUseCase::new(lazy_locator.get_plugin_registry().await)
            .execute(plugin_id)
            .await?,
    )
}

#[tracing::instrument]
pub async fn enable(plugin_id: String) -> crate::Result<()> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(EnablePluginUseCase::new(
        lazy_locator.get_plugin_registry().await,
        lazy_locator.get_plugin_loader_registry().await,
        lazy_locator.get_plugin_settings_storage().await,
        lazy_locator.get_settings_storage().await,
    )
    .execute(plugin_id)
    .await?)
}

#[tracing::instrument]
pub async fn disable(plugin_id: String) -> crate::Result<()> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(DisablePluginUseCase::new(
        lazy_locator.get_plugin_registry().await,
        lazy_locator.get_plugin_loader_registry().await,
        lazy_locator.get_settings_storage().await,
    )
    .execute(plugin_id)
    .await?)
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
pub async fn get_settings(plugin_id: String) -> crate::Result<Option<PluginSettings>> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        GetPluginSettingsUseCase::new(lazy_locator.get_plugin_settings_storage().await)
            .execute(plugin_id)
            .await?,
    )
}

#[tracing::instrument]
pub async fn edit_settings(
    plugin_id: String,
    edit_settings: EditPluginSettings,
) -> crate::Result<()> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        EditPluginSettingsUseCase::new(lazy_locator.get_plugin_settings_storage().await)
            .execute(plugin_id, edit_settings)
            .await?,
    )
}
