use async_trait::async_trait;

use crate::features::{
    events::{emit_instance, InstancePayloadType},
    instance::{Instance, InstanceManager, InstanceStorage},
};

pub struct InstanceManagerImpl<IS>
where
    IS: InstanceStorage,
{
    instance_storage: IS,
}

impl<IS> InstanceManagerImpl<IS>
where
    IS: InstanceStorage + Send + Sync,
{
    pub fn new(instance_storage: IS) -> Self {
        Self { instance_storage }
    }
}

#[async_trait]
impl<IS> InstanceManager for InstanceManagerImpl<IS>
where
    IS: InstanceStorage + Send + Sync,
{
    async fn list(&self) -> crate::Result<Vec<Instance>> {
        Ok(self.instance_storage.list().await?)
    }

    async fn get(&self, id: &str) -> crate::Result<Instance> {
        Ok(self.instance_storage.get(id).await?)
    }

    async fn upsert(&self, instance: &Instance) -> crate::Result<()> {
        self.instance_storage.upsert(instance).await?;
        emit_instance(&instance.id, InstancePayloadType::Edited).await?;
        Ok(())
    }

    async fn upsert_with<F>(&self, id: &str, update_fn: F) -> crate::Result<()>
    where
        F: FnOnce(&mut Instance) -> crate::Result<()> + Send,
    {
        let mut instance = self.instance_storage.get(id).await?;
        update_fn(&mut instance)?;
        self.upsert(&instance).await?;
        Ok(())
    }

    async fn remove(&self, id: &str) -> crate::Result<()> {
        self.instance_storage.remove(id).await?;
        emit_instance(id, InstancePayloadType::Removed).await
    }
}
