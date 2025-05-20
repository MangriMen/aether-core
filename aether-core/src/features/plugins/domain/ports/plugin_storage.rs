use std::collections::HashMap;

use async_trait::async_trait;

use crate::features::plugins::Plugin;

#[async_trait]
pub trait PluginStorage: Send + Sync {
    async fn list(&self) -> crate::Result<HashMap<String, Plugin>>;
    async fn get(&self, plugin_id: &str) -> crate::Result<Plugin>;
}
