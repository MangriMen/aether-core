use serde::{Deserialize, Serialize};

use crate::features::plugins::ImporterCapability;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Importer {
    pub plugin_id: String,
    pub capability: ImporterCapability,
}
