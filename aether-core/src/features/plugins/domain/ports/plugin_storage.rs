use std::collections::HashMap;

use async_trait::async_trait;

use crate::features::plugins::{Plugin, PluginError};

#[async_trait]
pub trait PluginStorage: Send + Sync {
    async fn list(&self) -> Result<HashMap<String, Plugin>, PluginError>;
    async fn get(&self, plugin_id: &str) -> Result<Plugin, PluginError>;
}
