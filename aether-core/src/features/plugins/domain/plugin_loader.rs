use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use super::{Plugin, PluginInstance, PluginSettings};

#[async_trait]
pub trait PluginLoader {
    async fn load(
        &self,
        plugin: &Plugin,
        settings: &Option<PluginSettings>,
    ) -> crate::Result<PluginInstance>;
    async fn unload(&self, instance: Arc<Mutex<PluginInstance>>) -> crate::Result<()>;
}
