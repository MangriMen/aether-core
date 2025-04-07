use std::process::ExitStatus;

use chrono::{DateTime, Utc};
use tokio::process::Command;
use uuid::Uuid;

use crate::{
    core::LauncherState,
    event::{emit_process, ProcessPayloadType},
    state::Instance,
    utils::io::IOError,
};

pub async fn manage_minecraft_process(
    id: String,
    post_exit_command: Option<String>,
    uuid: Uuid,
) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    let mut last_updated_playtime = Utc::now();
    let mc_exit_status = wait_for_process(&state, uuid, &id, &mut last_updated_playtime).await;

    state.process_manager.remove(uuid);
    emit_process(&id, uuid, ProcessPayloadType::Finished, "Exited process").await?;

    update_playtime_if_needed(&mut last_updated_playtime, &id, true).await;

    if mc_exit_status.success() {
        execute_post_exit_command(&id, post_exit_command).await?;
    }

    Ok(())
}

async fn update_playtime_if_needed(last_updated: &mut DateTime<Utc>, id: &str, force: bool) {
    let elapsed_seconds = Utc::now()
        .signed_duration_since(*last_updated)
        .num_seconds();

    if elapsed_seconds >= 60 || force {
        if let Err(e) = Instance::edit(id, |profile| {
            profile.time_played += elapsed_seconds as u64;
            async { Ok(()) }
        })
        .await
        {
            tracing::warn!("Failed to update playtime for profile {}: {}", id, e);
        }
        *last_updated = Utc::now();
    }
}

async fn wait_for_process(
    state: &LauncherState,
    uuid: Uuid,
    id: &str,
    last_updated: &mut DateTime<Utc>,
) -> ExitStatus {
    loop {
        if let Some(process) = state.process_manager.try_wait(uuid).unwrap_or(None) {
            if let Some(exit_status) = process {
                return exit_status;
            }
        } else {
            return ExitStatus::default();
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        update_playtime_if_needed(last_updated, id, false).await;
    }
}

async fn execute_post_exit_command(id: &str, command: Option<String>) -> crate::Result<()> {
    if let Some(hook) = command {
        let mut parts = hook.split_whitespace();
        if let Some(cmd) = parts.next() {
            let mut command = Command::new(cmd);
            command
                .args(parts.collect::<Vec<&str>>())
                .current_dir(Instance::get_full_path(id).await?);
            command.spawn().map_err(IOError::from)?;
        }
    }
    Ok(())
}
