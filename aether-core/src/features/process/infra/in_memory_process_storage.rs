use std::process::ExitStatus;

use async_trait::async_trait;
use dashmap::DashMap;
use uuid::Uuid;

use crate::features::process::{
    MinecraftProcess, MinecraftProcessMetadata, ProcessError, ProcessStorage,
};

#[derive(Debug, Default)]
pub struct InMemoryProcessStorage {
    processes: DashMap<Uuid, MinecraftProcess>,
}

#[async_trait]
impl ProcessStorage for InMemoryProcessStorage {
    async fn insert(&self, process: MinecraftProcess) {
        self.processes.insert(process.metadata.uuid, process);
    }

    async fn remove(&self, id: Uuid) {
        self.processes.remove(&id);
    }

    async fn list_metadata(&self) -> Vec<MinecraftProcessMetadata> {
        self.processes.iter().map(|x| x.metadata.clone()).collect()
    }

    async fn get_metadata(&self, id: Uuid) -> Option<MinecraftProcessMetadata> {
        self.processes.get(&id).map(|x| x.metadata.clone())
    }

    async fn try_wait(&self, id: Uuid) -> Result<Option<Option<ExitStatus>>, ProcessError> {
        if let Some(mut process) = self.processes.get_mut(&id) {
            return Ok(Some(
                process
                    .try_wait()
                    .map_err(|_| ProcessError::WaitError { id: id.to_string() })?,
            ));
        }
        Ok(None)
    }

    async fn wait_for(&self, id: Uuid) -> Result<(), ProcessError> {
        if let Some(mut process) = self.processes.get_mut(&id) {
            process
                .wait()
                .await
                .map_err(|_| ProcessError::WaitError { id: id.to_string() })?;
        }
        Ok(())
    }

    async fn kill(&self, id: Uuid) -> Result<(), ProcessError> {
        if let Some(mut process) = self.processes.get_mut(&id) {
            process
                .kill()
                .await
                .map_err(|_| ProcessError::KillError { id: id.to_string() })?;
        }
        Ok(())
    }
}
