use extism::FromBytes;
use extism_convert::{encoding, Msgpack};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, FromBytes)]
#[encoding(Msgpack)]
#[serde(rename_all = "camelCase")]
pub struct PackInfoDto {
    pub plugin_id: String,
    pub modpack_id: String,
    pub version: String,
}
