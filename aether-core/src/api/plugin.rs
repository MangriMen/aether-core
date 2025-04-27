use crate::{
    core::{domain::ServiceLocator, LauncherState},
    features::{
        plugins::{
            self, EditPluginSettings, FsPluginSettingsStorage, PluginManifest, PluginSettings,
        },
        settings::{FsSettingsStorage, SettingsStorage},
    },
};

fn get_settings_storage(state: &LauncherState) -> FsSettingsStorage {
    FsSettingsStorage::new(&state.locations.settings_dir)
}

async fn get_plugin_settings_storage() -> crate::Result<FsPluginSettingsStorage> {
    Ok(FsPluginSettingsStorage::new(
        LauncherState::get().await?.locations.clone(),
    ))
}

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

    Ok(plugin_service
        .list()
        .map(|value| value.manifest.clone())
        .collect())
}

#[tracing::instrument]
pub async fn get(id: &str) -> crate::Result<PluginManifest> {
    let service_locator = ServiceLocator::get().await?;
    let plugin_service = service_locator.plugin_service.read().await;

    Ok(plugin_service.get(id)?.manifest.clone())
}

#[tracing::instrument]
pub async fn is_enabled(id: &str) -> crate::Result<bool> {
    let service_locator = ServiceLocator::get().await?;
    let plugin_service = service_locator.plugin_service.read().await;

    if let Ok(plugin) = plugin_service.get(id) {
        return Ok(plugin.is_loaded());
    }

    Ok(false)
}

#[tracing::instrument]
pub async fn enable(id: &str) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    let service_locator = ServiceLocator::get().await?;

    let mut plugin_service = service_locator.plugin_service.write().await;

    plugin_service.load_plugin(id).await?;

    let settings_storage = get_settings_storage(&state);
    let mut settings = settings_storage.get().await?;
    if !settings.enabled_plugins.contains(id) {
        settings.enabled_plugins.insert(id.to_owned());
        settings_storage.upsert(&settings).await?;
    }

    Ok(())
}

#[tracing::instrument]
pub async fn disable(id: &str) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    let service_locator = ServiceLocator::get().await?;

    let mut plugin_service = service_locator.plugin_service.write().await;

    plugin_service.unload_plugin(id).await?;

    let settings_storage = get_settings_storage(&state);
    let mut settings = settings_storage.get().await?;
    if !settings.enabled_plugins.contains(id) {
        settings.enabled_plugins.remove(id);
        settings_storage.upsert(&settings).await?;
    }

    Ok(())
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
pub async fn get_settings(id: &str) -> crate::Result<PluginSettings> {
    plugins::get_plugin_settings(&get_plugin_settings_storage().await?, id).await
}

#[tracing::instrument]
pub async fn edit_settings(id: &str, edit_settings: &EditPluginSettings) -> crate::Result<()> {
    plugins::edit_plugin_settings(&get_plugin_settings_storage().await?, id, edit_settings).await
}
