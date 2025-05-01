use std::process::ExitStatus;

use async_trait::async_trait;
use tokio::process::Command;
use uuid::Uuid;

use crate::features::process::MinecraftProcessMetadata;

#[async_trait]
pub trait ProcessManager {
    async fn insert_new_process(
        &self,
        instance_id: &str,
        mut mc_command: Command,
        post_exit_command: Option<String>,
    ) -> crate::Result<MinecraftProcessMetadata>;
    fn list(&self) -> Vec<MinecraftProcessMetadata>;
    fn get(&self, id: Uuid) -> Option<MinecraftProcessMetadata>;
    fn remove(&self, id: Uuid);

    fn try_wait(&self, id: Uuid) -> crate::Result<Option<Option<ExitStatus>>>;
    async fn wait_for(&self, id: Uuid) -> crate::Result<()>;
    async fn kill(&self, id: Uuid) -> crate::Result<()>;
}
