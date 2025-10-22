use serde::{Deserialize, Serialize};

use crate::features::plugins::ImporterCapability;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginImporters {
    pub plugin_id: String,
    pub importers: Vec<ImporterCapability>,
}
