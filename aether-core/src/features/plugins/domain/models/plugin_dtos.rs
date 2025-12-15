use extism::ToBytes;
use extism_convert::Json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, ToBytes)]
#[encoding(Json)]
#[serde(rename_all = "camelCase")]
pub struct PluginImportInstance {
    pub importer_id: String,
    pub path: String,
}
