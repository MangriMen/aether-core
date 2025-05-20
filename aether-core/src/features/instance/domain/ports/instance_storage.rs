use async_trait::async_trait;

use crate::{features::instance::Instance, shared::StorageError};

#[async_trait]
pub trait InstanceStorage: Send + Sync {
    async fn list(&self) -> Result<Vec<Instance>, StorageError>;
    async fn get(&self, id: &str) -> Result<Instance, StorageError>;
    async fn upsert(&self, instance: &Instance) -> Result<(), StorageError>;
    async fn remove(&self, id: &str) -> Result<(), StorageError>;
}
