use extism::FromBytes;
use extism_convert::{encoding, Json};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, FromBytes)]
#[encoding(Json)]
#[serde(rename_all = "camelCase")]
pub struct PackInfo {
    pub modpack_id: String,
    pub version: String,
}
