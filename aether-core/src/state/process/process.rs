use std::process::ExitStatus;

use chrono::{DateTime, Utc};
use tokio::process::{Child, Command};
use uuid::Uuid;

use crate::{
    event::{emit_process, ProcessPayloadType},
    state::{Instance, LauncherState},
    utils::io::IOError,
};

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct MinecraftProcessMetadata {
    pub uuid: Uuid,
    pub name_id: String,
    pub start_time: DateTime<Utc>,
}

#[derive(Debug)]
pub struct MinecraftProcess {
    pub metadata: MinecraftProcessMetadata,
    pub child: Child,
}

impl MinecraftProcess {
    // Spawns a new child process and inserts it into the hashmap
    // Also, as the process ends, it spawns the follow-up process if it exists
    // By convention, ExitStatus is last command's exit status, and we exit on the first non-zero exit status
    pub async fn sequential_process_manager(
        name_id: String,
        post_exit_command: Option<String>,
        uuid: Uuid,
    ) -> anyhow::Result<()> {
        async fn update_playtime(
            last_updated_playtime: &mut DateTime<Utc>,
            name_id: &str,
            force_update: bool,
        ) {
            let diff = Utc::now()
                .signed_duration_since(*last_updated_playtime)
                .num_seconds();
            // if diff >= 60 || force_update {
            //     if let Err(e) = Instance::edit(profile_path, |mut prof| {
            //         prof.time_played += diff as u64;
            //         async { Ok(()) }
            //     })
            //     .await
            //     {
            //         tracing::warn!(
            //             "Failed to update playtime for profile {}: {}",
            //             &profile_path,
            //             e
            //         );
            //     }
            //     *last_updated_playtime = Utc::now();
            // }
        }

        // Wait on current Minecraft Child
        let mc_exit_status;
        let mut last_updated_playtime = Utc::now();

        let state = LauncherState::get().await?;
        loop {
            if let Some(process) = state.process_manager.try_wait(uuid)? {
                if let Some(t) = process {
                    mc_exit_status = t;
                    break;
                }
            } else {
                mc_exit_status = ExitStatus::default();
                break;
            }

            // sleep for 10ms
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

            // Auto-update playtime every minute
            update_playtime(&mut last_updated_playtime, &name_id, false).await;
        }

        state.process_manager.remove(uuid);
        emit_process(
            &name_id,
            uuid,
            ProcessPayloadType::Finished,
            "Exited process",
        )
        .await?;

        // Now fully complete- update playtime one last time
        update_playtime(&mut last_updated_playtime, &name_id, true).await;

        // Publish play time update
        // Allow failure, it will be stored locally and sent next time
        // Sent in another thread as first call may take a couple seconds and hold up process ending
        let profile = name_id.clone();
        // tokio::spawn(async move {
        //     if let Err(e) = Instance::try_update_playtime(&profile).await {
        //         tracing::warn!("Failed to update playtime for profile {}: {}", profile, e);
        //     }
        // });

        // let _ = state.discord_rpc.clear_to_default(true).await;

        // If in tauri, window should show itself again after process exists if it was hidden
        #[cfg(feature = "tauri")]
        {
            let window = crate::EventState::get_main_window().await?;
            if let Some(window) = window {
                window.unminimize()?;
                window.set_focus()?;
            }
        }

        if mc_exit_status.success() {
            // We do not wait on the post exist command to finish running! We let it spawn + run on its own.
            // This behaviour may be changed in the future
            if let Some(hook) = post_exit_command {
                let mut cmd = hook.split(' ');
                if let Some(command) = cmd.next() {
                    let mut command = Command::new(command);
                    command
                        .args(cmd.collect::<Vec<&str>>())
                        .current_dir(Instance::get_full_path(&name_id).await?);
                    command.spawn().map_err(IOError::from)?;
                }
            }
        }

        Ok(())
    }
}
