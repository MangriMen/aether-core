use async_trait::async_trait;

use super::Instance;

#[async_trait]
pub trait InstanceManager {
    async fn list(&self) -> crate::Result<Vec<Instance>>;
    async fn get(&self, id: &str) -> crate::Result<Instance>;
    async fn upsert(&self, instance: &Instance) -> crate::Result<()>;
    async fn upsert_with<F>(&self, instance_id: &str, update_fn: F) -> crate::Result<()>
    where
        F: FnOnce(&mut Instance) -> crate::Result<()> + Send;
    async fn remove(&self, id: &str) -> crate::Result<()>;
}
