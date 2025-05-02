use std::process::ExitStatus;

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::process::{MinecraftProcess, MinecraftProcessMetadata};

#[async_trait]
pub trait ProcessStorage {
    async fn insert(&self, process: MinecraftProcess);
    async fn remove(&self, id: Uuid);

    async fn list_metadata(&self) -> Vec<MinecraftProcessMetadata>;
    async fn get_metadata(&self, id: Uuid) -> Option<MinecraftProcessMetadata>;

    async fn try_wait(&self, id: Uuid) -> crate::Result<Option<Option<ExitStatus>>>;
    async fn wait_for(&self, id: Uuid) -> crate::Result<()>;
    async fn kill(&self, id: Uuid) -> crate::Result<()>;
}
