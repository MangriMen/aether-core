use crate::{core::LauncherState, features::instance::ImportConfig};

pub async fn get_import_configs() -> crate::Result<Vec<ImportConfig>> {
    let state = LauncherState::get().await?;
    let plugin_manager = state.plugin_manager.read().await;

    let mut import_handlers: Vec<ImportConfig> = Vec::new();

    for plugin_state in plugin_manager.get_plugins() {
        if let Some(plugin) = plugin_state.get_plugin() {
            let mut plugin = plugin.lock().await;
            if plugin.supports_get_import_config() {
                if let Ok(import_config) = plugin.get_import_config() {
                    import_handlers.push(import_config);
                }
            }
        }
    }

    Ok(import_handlers)
}

pub async fn import(pack_type: &str, path_or_url: &str) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    let plugin_manager = state.plugin_manager.read().await;

    if let Ok(plugin) = plugin_manager.get_plugin(pack_type) {
        if let Some(plugin) = plugin.get_plugin() {
            plugin.lock().await.import(path_or_url).map_err(|_| {
                crate::ErrorKind::InstanceImportError(format!(
                    "Failed to import instance from plugin {pack_type}"
                ))
                .as_error()
            })?;
        }

        Ok(())
    } else {
        Err(crate::ErrorKind::InstanceImportError("Unsupported pack type".to_owned()).as_error())
    }
}
