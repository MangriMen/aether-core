use extism::ToBytes;
use extism_convert::{encoding, Json};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, ToBytes)]
#[encoding(Json)]
pub enum PluginEvent {
    BeforeInstanceLaunch { instance_id: String },
    AfterInstanceLaunch { instance_id: String },
}
