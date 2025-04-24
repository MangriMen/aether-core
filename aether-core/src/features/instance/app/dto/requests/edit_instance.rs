use extism::{FromBytes, ToBytes};
use extism_convert::{encoding, Json};
use serde::{Deserialize, Serialize};

use crate::features::settings::{MemorySettings, WindowSize};

#[derive(Debug, Serialize, Deserialize, FromBytes, ToBytes)]
#[encoding(Json)]
#[serde(rename_all = "camelCase")]
pub struct EditInstance {
    pub name: Option<String>,
    pub java_path: Option<Option<String>>,
    pub extra_launch_args: Option<Option<Vec<String>>>,
    pub custom_env_vars: Option<Option<Vec<(String, String)>>>,
    pub memory: Option<Option<MemorySettings>>,
    pub game_resolution: Option<Option<WindowSize>>,
}
