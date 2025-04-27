use std::collections::HashMap;

use async_trait::async_trait;

use super::Plugin;

#[async_trait]
pub trait PluginStorage {
    async fn list(&self) -> crate::Result<HashMap<String, Plugin>>;
    async fn get(&self, plugin_id: &str) -> crate::Result<Plugin>;
}
