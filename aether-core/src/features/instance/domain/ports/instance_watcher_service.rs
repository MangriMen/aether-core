use async_trait::async_trait;

#[async_trait]
pub trait InstanceWatcherService: Send + Sync {
    async fn watch_instances(&self) -> crate::Result<()>;
    async fn watch_instance(&self, instance_id: &str) -> crate::Result<()>;
    async fn unwatch_instance(&self, instance_id: &str) -> crate::Result<()>;
}
