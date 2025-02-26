use std::path::{Path, PathBuf};

use crate::state::{settings, LauncherState, PluginMetadata, PluginSettings};

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

    Ok(())
}

#[tracing::instrument]
pub async fn disable(id: &str) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    let mut plugin_manager = state.plugin_manager.write().await;

    plugin_manager.unload_plugin(id).await?;

    Ok(())
}

#[tracing::instrument]
pub async fn call(id: &str, data: &str) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    let plugin_manager = state.plugin_manager.read().await;

    let plugin = plugin_manager.get_plugin(id)?;

    log::debug!("Calling plugin {:?}", id);
    // plugin.plugin.call(data).await?;

    Ok(())
}

#[tracing::instrument]
pub async fn get_settings(id: &str) -> crate::Result<PluginSettings> {
    let state = LauncherState::get().await?;

    if !state.locations.plugin_settings(id).exists() {
        return Ok(PluginSettings::default());
    }

    crate::utils::io::read_toml_async(state.locations.plugin_settings(id)).await
}

#[tracing::instrument]
pub async fn edit_settings(id: &str, settings: &PluginSettings) -> crate::Result<()> {
    let state = LauncherState::get().await?;

    let mut current_settings: PluginSettings = get_settings(id).await?;

    if let Some(allowed_hosts) = &settings.allowed_hosts {
        current_settings.allowed_hosts = Some(allowed_hosts.to_vec());
    }

    if let Some(allowed_paths) = &settings.allowed_paths {
        current_settings.allowed_paths = Some(
            allowed_paths
                .iter()
                .filter(|(from, _)| Path::new(from).exists())
                .cloned()
                .collect::<Vec<(String, PathBuf)>>(),
        );
    }

    crate::utils::io::write_toml_async(state.locations.plugin_settings(id), &current_settings)
        .await?;

    Ok(())
}
