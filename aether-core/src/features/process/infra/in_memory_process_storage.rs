use std::process::ExitStatus;

use async_trait::async_trait;
use dashmap::DashMap;
use uuid::Uuid;

use crate::features::process::{MinecraftProcess, MinecraftProcessMetadata, ProcessStorage};

#[derive(Debug, Default)]
pub struct InMemoryProcessStorage {
    processes: DashMap<Uuid, MinecraftProcess>,
}

#[async_trait]
impl ProcessStorage for InMemoryProcessStorage {
    async fn insert(&self, process: MinecraftProcess) {
        self.processes.insert(process.metadata.uuid, process);
    }

    async fn list_metadata(&self) -> Vec<MinecraftProcessMetadata> {
        self.processes
            .iter()
            .map(|x| x.value().metadata.clone())
            .collect()
    }

    async fn get_metadata(&self, id: Uuid) -> Option<MinecraftProcessMetadata> {
        self.processes.get(&id).map(|x| x.metadata.clone())
    }

    async fn try_wait(&self, id: Uuid) -> crate::Result<Option<Option<ExitStatus>>> {
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

    async fn remove(&self, id: Uuid) {
        self.processes.remove(&id);
    }
}
