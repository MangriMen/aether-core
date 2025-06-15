use std::sync::Arc;
use tokio::sync::Mutex;

use crate::features::plugins::PluginInstance;

use super::PluginManifest;

#[derive(Clone)]
pub struct Plugin {
    pub manifest: PluginManifest,
    pub hash: String,
    pub instance: Option<Arc<Mutex<dyn PluginInstance>>>,
}

impl Plugin {
    pub fn is_loaded(&self) -> bool {
        self.instance.is_some()
    }
}
