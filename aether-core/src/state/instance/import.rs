use extism::{FromBytes, ToBytes};
use extism_convert::{encoding, Json};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, FromBytes, ToBytes)]
#[encoding(Json)]
#[serde(rename_all = "camelCase")]
pub struct ImportConfig {
    pub pack_type: String,
    pub title: String,
    pub field_label: String,
    pub file_name: String,
    pub file_extensions: Vec<String>,
}
