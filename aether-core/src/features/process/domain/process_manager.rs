use std::process::ExitStatus;

use chrono::Utc;
use dashmap::DashMap;
use tokio::process::Command;
use uuid::Uuid;

use crate::{
    features::{
        events::{emit_process, ProcessPayloadType},
        process::manage_minecraft_process,
    },
    shared::IOError,
};

use super::{MinecraftProcess, MinecraftProcessMetadata};

#[derive(Debug, Default)]
pub struct ProcessManager {
    processes: DashMap<Uuid, MinecraftProcess>,
}

impl ProcessManager {
    pub async fn insert_new_process(
        &self,
        profile_path: &str,
        mut mc_command: Command,
        post_exit_command: Option<String>,
    ) -> crate::Result<MinecraftProcessMetadata> {
        let mc_proc = mc_command.spawn().map_err(IOError::from)?;

        let process = MinecraftProcess {
            metadata: MinecraftProcessMetadata {
                uuid: Uuid::new_v4(),
                start_time: Utc::now(),
                id: profile_path.to_string(),
            },
            child: mc_proc,
        };

        let metadata = process.metadata.clone();

        tokio::spawn(manage_minecraft_process(
            profile_path.to_string(),
            post_exit_command,
            metadata.uuid,
        ));

        self.processes.insert(process.metadata.uuid, process);

        emit_process(
            profile_path,
            metadata.uuid,
            ProcessPayloadType::Launched,
            "Launched Minecraft",
        )
        .await?;

        Ok(metadata)
    }

    pub fn list(&self) -> Vec<MinecraftProcessMetadata> {
        self.processes
            .iter()
            .map(|x| x.value().metadata.clone())
            .collect()
    }

    pub fn get(&self, id: Uuid) -> Option<MinecraftProcessMetadata> {
        self.processes.get(&id).map(|x| x.metadata.clone())
    }

    pub fn try_wait(&self, id: Uuid) -> crate::Result<Option<Option<ExitStatus>>> {
        if let Some(mut process) = self.processes.get_mut(&id) {
            Ok(Some(process.child.try_wait()?))
        } else {
            Ok(None)
        }
    }

    pub async fn wait_for(&self, id: Uuid) -> crate::Result<()> {
        if let Some(mut process) = self.processes.get_mut(&id) {
            process.child.wait().await?;
        }
        Ok(())
    }

    pub async fn kill(&self, id: Uuid) -> crate::Result<()> {
        if let Some(mut process) = self.processes.get_mut(&id) {
            process.child.kill().await?;
        }

        Ok(())
    }

    pub fn remove(&self, id: Uuid) {
        self.processes.remove(&id);
    }
}
