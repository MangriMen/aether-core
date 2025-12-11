use extism::FromBytes;
use extism_convert::{encoding, Json};
use register_schema_derive::RegisterSchema;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, RegisterSchema, FromBytes)]
#[encoding(Json)]
#[serde(rename_all = "camelCase")]
pub struct PackInfo {
    pub plugin_id: String,
    pub modpack_id: String,
    pub version: String,
}
