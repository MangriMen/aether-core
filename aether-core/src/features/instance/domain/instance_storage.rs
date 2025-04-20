use async_trait::async_trait;

use crate::shared::StorageError;

use super::Instance;

#[async_trait]
pub trait InstanceStorage {
    async fn get(&self, id: &str) -> Result<Instance, StorageError>;
    async fn upsert(&self, instance: &Instance) -> Result<(), StorageError>;
    async fn remove(&self, id: &str) -> Result<(), StorageError>;
}
