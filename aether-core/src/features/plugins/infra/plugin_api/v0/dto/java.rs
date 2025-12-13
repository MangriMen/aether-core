use extism::ToBytes;
use extism_convert::{encoding, Msgpack};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, ToBytes)]
#[encoding(Msgpack)]
pub struct JavaDto {
    pub major_version: u32,
    pub version: String,
    pub architecture: String,
    pub path: String,
}
