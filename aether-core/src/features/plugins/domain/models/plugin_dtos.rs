use extism::ToBytes;
use extism_convert::Msgpack;
use serde::{Deserialize, Serialize};

use crate::features::instance::ContentInstallParams;

#[derive(Debug, Clone, Serialize, Deserialize, ToBytes)]
#[encoding(Msgpack)]
#[serde(rename_all = "camelCase")]
pub struct PluginImportInstance {
    pub importer_id: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToBytes)]
#[encoding(Msgpack)]
#[serde(rename_all = "camelCase")]
pub struct PluginInstallContent {
    pub instance_id: String,
    pub install_params: ContentInstallParams,
}
