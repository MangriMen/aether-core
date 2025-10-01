use extism::{FromBytes, ToBytes};
use extism_convert::{encoding, Json};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, FromBytes, ToBytes)]
#[encoding(Json)]
#[serde(rename_all = "camelCase")]
pub struct ImportConfig {
    pub plugin_id: String,
    pub title: String,
    pub field_label: String,
    pub file_name: String,
    pub file_extensions: Vec<String>,
}
