use crate::state::LauncherState;

pub async fn call(id: &str, data: &str) -> crate::Result<()> {
    log::debug!("Calling plugin {:?} with data {:?}", id, data);

    let state = LauncherState::get().await?;

    match state.plugins.get(id) {
        Some(plugin) => plugin.call(data).await?,
        None => {
            return Err(
                crate::ErrorKind::PluginNotFoundError("Plugin not found".to_string()).as_error(),
            )
        }
    }

    Ok(())
}
