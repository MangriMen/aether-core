use crate::{
    features::instance::{Instance, InstanceStorage},
    shared::StorageError,
};
use async_trait::async_trait;

#[async_trait]
pub trait InstanceStorageExt: InstanceStorage {
    async fn upsert_with<F>(&self, id: &str, update_fn: F) -> Result<(), StorageError>
    where
        F: FnOnce(&mut Instance) -> Result<(), StorageError> + Send;
}

#[async_trait]
impl<IS: InstanceStorage> InstanceStorageExt for IS {
    async fn upsert_with<F>(&self, id: &str, update_fn: F) -> Result<(), StorageError>
    where
        F: FnOnce(&mut Instance) -> Result<(), StorageError> + Send,
    {
        let mut instance = self.get(id).await?;
        update_fn(&mut instance)?;
        self.upsert(&instance).await
    }
}
