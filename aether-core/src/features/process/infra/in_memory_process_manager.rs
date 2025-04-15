use std::process::ExitStatus;

use async_trait::async_trait;
use dashmap::DashMap;
use tokio::process::Command;
use uuid::Uuid;

use crate::{
    features::{
        events::{emit_process, ProcessPayloadType},
        process::{
            manage_minecraft_process, MinecraftProcess, MinecraftProcessMetadata, ProcessManager,
        },
    },
    shared::IOError,
};

#[derive(Debug, Default)]
pub struct InMemoryProcessManager {
    processes: DashMap<Uuid, MinecraftProcess>,
}

#[async_trait]
impl ProcessManager for InMemoryProcessManager {
    async fn insert_new_process(
        &self,
        instance_id: &str,
        mut mc_command: Command,
        post_exit_command: Option<String>,
    ) -> crate::Result<MinecraftProcessMetadata> {
        let minecraft_process = mc_command.spawn().map_err(IOError::from)?;
        let process = MinecraftProcess::from_child(instance_id, minecraft_process);

        let metadata = process.metadata.clone();
        tokio::spawn(manage_minecraft_process(
            instance_id.to_string(),
            post_exit_command,
            metadata.uuid,
        ));
        self.processes.insert(process.metadata.uuid, process);

        emit_process(
            instance_id,
            metadata.uuid,
            ProcessPayloadType::Launched,
            "Launched Minecraft",
        )
        .await?;

        Ok(metadata)
    }

    fn list(&self) -> Vec<MinecraftProcessMetadata> {
        self.processes
            .iter()
            .map(|x| x.value().metadata.clone())
            .collect()
    }

    fn get(&self, id: Uuid) -> Option<MinecraftProcessMetadata> {
        self.processes.get(&id).map(|x| x.metadata.clone())
    }

    fn try_wait(&self, id: Uuid) -> crate::Result<Option<Option<ExitStatus>>> {
        if let Some(mut process) = self.processes.get_mut(&id) {
            Ok(Some(process.child.try_wait()?))
        } else {
            Ok(None)
        }
    }

    async fn wait_for(&self, id: Uuid) -> crate::Result<()> {
        if let Some(mut process) = self.processes.get_mut(&id) {
            process.child.wait().await?;
        }
        Ok(())
    }

    async fn kill(&self, id: Uuid) -> crate::Result<()> {
        if let Some(mut process) = self.processes.get_mut(&id) {
            process.child.kill().await?;
        }
        Ok(())
    }

    fn remove(&self, id: Uuid) {
        self.processes.remove(&id);
    }
}
