use extism::{FromBytes, ToBytes};
use extism_convert::{encoding, Msgpack};
use serde::{Deserialize, Serialize};

use crate::features::plugins::v0::{LoaderVersionPreferenceDto, ModLoaderDto, PackInfoDto};

#[derive(Serialize, Deserialize, FromBytes, ToBytes)]
#[encoding(Msgpack)]
#[serde(rename_all = "camelCase")]
pub struct NewInstanceDto {
    pub name: String,
    pub game_version: String,
    pub mod_loader: ModLoaderDto,
    pub loader_version: Option<LoaderVersionPreferenceDto>,
    pub icon_path: Option<String>,
    pub skip_install_instance: Option<bool>,
    pub pack_info: Option<PackInfoDto>,
}
