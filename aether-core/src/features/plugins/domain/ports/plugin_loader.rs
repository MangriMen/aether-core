use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::features::plugins::{Plugin, PluginError, PluginSettings};

use super::PluginInstance;

#[async_trait]
pub trait PluginLoader: Send + Sync {
    async fn load(
        &self,
        plugin: &Plugin,
        settings: &Option<PluginSettings>,
    ) -> Result<Arc<Mutex<dyn PluginInstance>>, PluginError>;
    async fn unload(&self, instance: Arc<Mutex<dyn PluginInstance>>) -> Result<(), PluginError>;
}
