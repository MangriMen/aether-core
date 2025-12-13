use crate::features::{instance::PackInfo, plugins::v0::PackInfoDto};

impl From<PackInfoDto> for PackInfo {
    fn from(value: PackInfoDto) -> Self {
        Self {
            plugin_id: value.plugin_id,
            modpack_id: value.modpack_id,
            version: value.version,
        }
    }
}
