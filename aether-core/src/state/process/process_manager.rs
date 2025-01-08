use std::process::ExitStatus;

use chrono::Utc;
use dashmap::DashMap;
use tokio::process::Command;
use uuid::Uuid;

use crate::{
    event::{emit_process, ProcessPayloadType},
    utils::io::IOError,
};

use super::{MinecraftProcess, MinecraftProcessMetadata};

#[derive(Debug)]
pub struct ProcessManager {
    processes: DashMap<Uuid, MinecraftProcess>,
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            processes: DashMap::new(),
        }
    }

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

        tokio::spawn(MinecraftProcess::sequential_process_manager(
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

    pub fn get(&self, id: Uuid) -> Option<MinecraftProcessMetadata> {
        self.processes.get(&id).map(|x| x.metadata.clone())
    }

    pub fn get_all(&self) -> Vec<MinecraftProcessMetadata> {
        self.processes
            .iter()
            .map(|x| x.value().metadata.clone())
            .collect()
    }

    pub fn try_wait(&self, id: Uuid) -> anyhow::Result<Option<Option<ExitStatus>>> {
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
