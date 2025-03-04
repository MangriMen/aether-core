use extism::ToBytes;
use extism_convert::{encoding, Json};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, ToBytes)]
#[encoding(Json)]
pub enum PluginEvent {
    BeforeInstanceLaunch { instance_id: String },
    AfterInstanceLaunch { instance_id: String },
}
