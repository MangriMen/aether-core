use async_trait::async_trait;
use log::error;

use crate::{
    features::{
        events::{emit_instance, InstancePayloadType},
        instance::{Instance, InstanceStorage},
    },
    shared::StorageError,
};

pub struct EventEmittingInstanceStorage<IS>
where
    IS: InstanceStorage,
{
    instance_storage: IS,
}

impl<IS> EventEmittingInstanceStorage<IS>
where
    IS: InstanceStorage + Send + Sync,
{
    pub fn new(instance_storage: IS) -> Self {
        Self { instance_storage }
    }
}

#[async_trait]
impl<IS> InstanceStorage for EventEmittingInstanceStorage<IS>
where
    IS: InstanceStorage + Send + Sync,
{
    async fn list(&self) -> Result<Vec<Instance>, StorageError> {
        Ok(self.instance_storage.list().await?)
    }

    async fn get(&self, id: &str) -> Result<Instance, StorageError> {
        Ok(self.instance_storage.get(id).await?)
    }

    async fn upsert(&self, instance: &Instance) -> Result<(), StorageError> {
        self.instance_storage.upsert(instance).await?;
        if let Err(e) = emit_instance(&instance.id, InstancePayloadType::Edited).await {
            error!("Failed to emit event: {}", e);
        }

        Ok(())
    }

    async fn remove(&self, id: &str) -> Result<(), StorageError> {
        self.instance_storage.remove(id).await?;
        if let Err(e) = emit_instance(id, InstancePayloadType::Removed).await {
            error!("Failed to emit event: {}", e);
        }

        Ok(())
    }
}
