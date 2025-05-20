use std::path::PathBuf;

use crate::core::LauncherState;

#[tracing::instrument]
pub async fn get_dir(id: &str) -> crate::Result<PathBuf> {
    let state = LauncherState::get().await?;
    Ok(state.location_info.instance_dir(id))
}
