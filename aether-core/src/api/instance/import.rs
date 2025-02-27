use crate::state::ImportHandler;

pub async fn get_import_handlers() -> crate::Result<Vec<ImportHandler>> {
    Ok(vec![ImportHandler {
        pack_type: "packwiz".to_string(),
        title: "Packwiz".to_string(),
        field_label: "Packwiz pack URL or file".to_string(),
        file_name: "Packwiz modpack".to_string(),
        file_extensions: vec!["toml".to_string()],
    }])
}

pub async fn import(pack_type: &str, path_or_url: &str) -> crate::Result<()> {
    match pack_type {
        "packwiz" => {
            let state = crate::state::LauncherState::get().await?;
            let mut plugin_manager = state.plugin_manager.write().await;

            if let Ok(plugin) = plugin_manager.get_plugin_mut(pack_type) {
                if let Some(plugin) = plugin.get_plugin() {
                    plugin
                        .lock()
                        .await
                        .import(path_or_url.to_owned())
                        .map_err(|_| {
                            crate::ErrorKind::InstanceImportError(format!(
                                "Failed to import instance from plugin {pack_type}"
                            ))
                            .as_error()
                        })?;
                }
            }

            Ok(())
        }
        _ => Err(
            crate::ErrorKind::InstanceImportError("Unsupported pack type".to_owned()).as_error(),
        ),
    }
}
