use crate::{
    core::LauncherState,
    features::{
        plugins::{self, FsPluginSettingsStorage, PluginMetadata, PluginSettings},
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
    let state = LauncherState::get().await?;
    let mut plugin_manager = state.plugin_manager.write().await;

    plugin_manager
        .scan_plugins(&state.locations.plugins_dir())
        .await
}

#[tracing::instrument]
pub async fn list() -> crate::Result<Vec<PluginMetadata>> {
    let state = LauncherState::get().await?;
    let plugin_manager = state.plugin_manager.read().await;

    Ok(plugin_manager
        .get_plugins()
        .map(|value| value.metadata.clone())
        .collect())
}

#[tracing::instrument]
pub async fn get(id: &str) -> crate::Result<PluginMetadata> {
    let state = LauncherState::get().await?;
    let plugin_manager = state.plugin_manager.read().await;

    Ok(plugin_manager.get_plugin(id)?.metadata.clone())
}

#[tracing::instrument]
pub async fn is_enabled(id: &str) -> crate::Result<bool> {
    let state = LauncherState::get().await?;
    let plugin_manager = state.plugin_manager.read().await;

    if let Ok(plugin) = plugin_manager.get_plugin(id) {
        return Ok(plugin.is_loaded());
    }

    Ok(false)
}

#[tracing::instrument]
pub async fn enable(id: &str) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    let mut plugin_manager = state.plugin_manager.write().await;

    plugin_manager.load_plugin(id).await?;

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
    let mut plugin_manager = state.plugin_manager.write().await;

    plugin_manager.unload_plugin(id).await?;

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
    let state = LauncherState::get().await?;
    let plugin_manager = state.plugin_manager.read().await;

    let _plugin = plugin_manager.get_plugin(id)?;

    log::debug!("Calling plugin {:?}", id);
    // plugin.plugin.call(data).await?;

    Ok(())
}

#[tracing::instrument]
pub async fn get_settings(id: &str) -> crate::Result<PluginSettings> {
    plugins::get_plugin_settings(&get_plugin_settings_storage().await?, id).await
}

#[tracing::instrument]
pub async fn edit_settings(id: &str, new_settings: &PluginSettings) -> crate::Result<()> {
    plugins::edit_plugin_settings(&get_plugin_settings_storage().await?, id, new_settings).await
}
