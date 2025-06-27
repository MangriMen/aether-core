use std::process::ExitStatus;

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::process::{MinecraftProcess, MinecraftProcessMetadata, ProcessError};

#[async_trait]
pub trait ProcessStorage: Send + Sync {
    async fn insert(&self, process: MinecraftProcess);
    async fn remove(&self, id: Uuid);

    async fn list_metadata(&self) -> Vec<MinecraftProcessMetadata>;
    async fn get_metadata(&self, id: Uuid) -> Option<MinecraftProcessMetadata>;

    async fn try_wait(&self, id: Uuid) -> Result<Option<Option<ExitStatus>>, ProcessError>;
    async fn wait_for(&self, id: Uuid) -> Result<(), ProcessError>;
    async fn kill(&self, id: Uuid) -> Result<(), ProcessError>;
}
