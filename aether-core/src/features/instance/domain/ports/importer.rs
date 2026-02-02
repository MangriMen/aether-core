use async_trait::async_trait;

use crate::features::instance::{ImporterCapability, InstanceError};

#[async_trait]
pub trait Importer: Send + Sync {
    fn info(&self) -> &ImporterCapability;

    async fn import(&self, path: &str) -> Result<(), InstanceError>;
}
