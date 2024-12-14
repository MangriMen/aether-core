use uuid::Uuid;

use crate::state::{LauncherState, MinecraftProcessMetadata};

// Gets the Profile paths of each *running* stored process in the state
#[tracing::instrument]
pub async fn get_all() -> crate::Result<Vec<MinecraftProcessMetadata>> {
    let state = LauncherState::get().await?;
    let processes = state.process_manager.get_all();
    Ok(processes)
}

// Gets the UUID of each stored process in the state by profile path
#[tracing::instrument]
pub async fn get_by_instance_name_id(
    name_id: &str,
) -> crate::Result<Vec<MinecraftProcessMetadata>> {
    let state = LauncherState::get().await?;

    let processes = state
        .process_manager
        .get_all()
        .into_iter()
        .filter(|x| x.name_id == name_id)
        .collect();

    Ok(processes)
}

// Kill a child process stored in the state by UUID, as a string
#[tracing::instrument]
pub async fn kill(uuid: Uuid) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    state.process_manager.kill(uuid).await?;

    Ok(())
}

// Wait for a child process stored in the state by UUID
#[tracing::instrument]
pub async fn wait_for(uuid: Uuid) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    state.process_manager.wait_for(uuid).await?;

    Ok(())
}
