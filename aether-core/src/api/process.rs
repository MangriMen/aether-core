use uuid::Uuid;

use crate::{
    core::LauncherState,
    features::process::{self, MinecraftProcessMetadata},
};

#[tracing::instrument]
pub async fn list() -> crate::Result<Vec<MinecraftProcessMetadata>> {
    let state = LauncherState::get().await?;
    Ok(process::list_process(&state.process_manager))
}

#[tracing::instrument]
pub async fn get_by_instance_id(id: &str) -> crate::Result<Vec<MinecraftProcessMetadata>> {
    let state = LauncherState::get().await?;
    Ok(process::get_process_by_instance_id(
        &state.process_manager,
        id,
    ))
}

#[tracing::instrument]
pub async fn kill(uuid: Uuid) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    process::kill_process(&state.process_manager, uuid).await
}

#[tracing::instrument]
pub async fn wait_for(uuid: Uuid) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    process::wait_for_process(&state.process_manager, uuid).await
}
