use async_trait::async_trait;

use crate::features::instance::{InstanceError, UpdaterCapability};

#[async_trait]
pub trait Updater: Send + Sync {
    fn info(&self) -> &UpdaterCapability;

    async fn update(&self, instance_id: &str) -> Result<(), InstanceError>;
}
