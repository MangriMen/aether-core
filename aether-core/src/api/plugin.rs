use crate::state::LauncherState;

pub async fn call(id: &str, data: &str) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    let plugin_manager = state.plugin_manager.read().await;

    let plugin = plugin_manager.get_plugin(id)?;

    log::debug!("Calling plugin {:?}", id);
    plugin.plugin.call(data).await?;

    Ok(())
}
