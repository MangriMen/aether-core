use std::process::ExitStatus;

use chrono::{DateTime, Utc};
use tokio::process::Command;
use uuid::Uuid;

use crate::{
    core::LauncherState,
    features::{
        events::{emit_process, ProcessPayloadType},
        instance::Instance,
        process::ProcessManager,
    },
    shared::IOError,
};

pub async fn manage_minecraft_process(
    id: String,
    post_exit_command: Option<String>,
    uuid: Uuid,
) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    let mut last_updated_playtime = Utc::now();

    let mc_exit_status =
        track_instance_process(&state, uuid, &id, &mut last_updated_playtime).await;

    state.process_manager.remove(uuid);
    emit_process(&id, uuid, ProcessPayloadType::Finished, "Exited process").await?;

    if mc_exit_status.success() {
        execute_post_exit_command(&id, post_exit_command).await?;
    }

    Ok(())
}

async fn update_playtime(last_updated: &mut DateTime<Utc>, id: &str, force: bool) {
    let elapsed_seconds = Utc::now()
        .signed_duration_since(*last_updated)
        .num_seconds();

    if elapsed_seconds >= 60 || force {
        let result = Instance::edit(id, |profile| {
            profile.time_played += elapsed_seconds as u64;
            async { Ok(()) }
        })
        .await;

        if let Err(e) = result {
            tracing::warn!("Failed to update playtime for profile {}: {}", id, e);
        }

        *last_updated = Utc::now();
    }
}

async fn track_instance_process(
    state: &LauncherState,
    uuid: Uuid,
    id: &str,
    last_updated: &mut DateTime<Utc>,
) -> ExitStatus {
    let check_threshold = tokio::time::Duration::from_millis(50);

    loop {
        match state.process_manager.try_wait(uuid) {
            Ok(Some(Some(exit_status))) => {
                // Process exited successfully
                update_playtime(last_updated, id, true).await;
                return exit_status;
            }
            Ok(Some(None)) => {} // Still running
            Ok(None) | Err(_) => {
                update_playtime(last_updated, id, true).await;
                return ExitStatus::default();
            }
        }

        tokio::time::sleep(check_threshold).await;
        update_playtime(last_updated, id, false).await;
    }
}

async fn execute_post_exit_command(id: &str, command: Option<String>) -> crate::Result<()> {
    let Some(command) = command else {
        return Ok(());
    };

    let mut split = command.split_whitespace();
    let Some(program) = split.next() else {
        return Ok(());
    };

    let working_dir = Instance::get_full_path(id).await?;

    Command::new(program)
        .args(split)
        .current_dir(working_dir)
        .spawn()
        .map_err(IOError::from)?;

    Ok(())
}
