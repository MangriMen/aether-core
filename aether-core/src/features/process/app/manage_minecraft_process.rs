use std::process::ExitStatus;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    core::LauncherState,
    features::{
        events::{emit_process, ProcessPayloadType},
        instance::{FsInstanceStorage, InstanceManager, InstanceManagerImpl},
        process::ProcessManager,
        settings::SerializableCommand,
    },
    shared::IOError,
};

pub async fn manage_minecraft_process(
    instance_id: String,
    post_exit_command: Option<String>,
    uuid: Uuid,
) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    let instance_storage =
        InstanceManagerImpl::new(FsInstanceStorage::new(state.locations.clone()));

    let mut last_updated_playtime = Utc::now();

    let mc_exit_status = track_instance_process(
        &state,
        &instance_storage,
        uuid,
        &instance_id,
        &mut last_updated_playtime,
    )
    .await;

    state.process_manager.remove(uuid);
    emit_process(
        &instance_id,
        uuid,
        ProcessPayloadType::Finished,
        "Exited process",
    )
    .await?;

    if mc_exit_status.success() {
        if let Some(command) = post_exit_command {
            let instance_dir = state.locations.instance_dir(&instance_id);
            if let Ok(cmd) = SerializableCommand::from_string(&command, Some(&instance_dir)) {
                cmd.to_tokio_command()
                    .spawn()
                    .map_err(|e| IOError::with_path(e, instance_dir))?;
            }
        }
    }

    Ok(())
}

async fn update_playtime<IM>(
    instance_manager: &IM,
    last_updated: &mut DateTime<Utc>,
    id: &str,
    force: bool,
) where
    IM: InstanceManager + ?Sized,
{
    let elapsed_seconds = Utc::now()
        .signed_duration_since(*last_updated)
        .num_seconds();

    if elapsed_seconds >= 60 || force {
        let result = async {
            instance_manager
                .upsert_with(id, |instance| {
                    instance.time_played += elapsed_seconds as u64;
                    Ok(())
                })
                .await
        }
        .await;

        if let Err(e) = result {
            tracing::warn!("Failed to update playtime for profile {}: {}", id, e);
        }

        *last_updated = Utc::now();
    }
}

async fn track_instance_process<IM>(
    state: &LauncherState,
    instance_manager: &IM,
    uuid: Uuid,
    id: &str,
    last_updated: &mut DateTime<Utc>,
) -> ExitStatus
where
    IM: InstanceManager + ?Sized,
{
    let check_threshold = tokio::time::Duration::from_millis(50);

    loop {
        match state.process_manager.try_wait(uuid) {
            Ok(Some(Some(exit_status))) => {
                // Process exited successfully
                update_playtime(instance_manager, last_updated, id, true).await;
                return exit_status;
            }
            Ok(Some(None)) => {} // Still running
            Ok(None) | Err(_) => {
                update_playtime(instance_manager, last_updated, id, true).await;
                return ExitStatus::default();
            }
        }

        tokio::time::sleep(check_threshold).await;
        update_playtime(instance_manager, last_updated, id, false).await;
    }
}
