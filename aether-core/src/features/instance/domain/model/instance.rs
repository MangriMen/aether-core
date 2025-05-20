use std::collections::HashMap;

use chrono::{DateTime, Utc};
use extism::{FromBytes, ToBytes};
use extism_convert::{encoding, Json};
use serde::{Deserialize, Serialize};

use crate::features::{
    minecraft::ModLoader,
    settings::{Hooks, MemorySettings, WindowSize},
};

use super::{ContentType, InstanceInstallStage};

#[derive(Serialize, Deserialize, Clone, Debug, FromBytes)]
#[encoding(Json)]
#[serde(rename_all = "camelCase")]
pub struct PackInfo {
    pub pack_type: String,
    pub pack_version: String,
    pub can_update: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    pub id: String,

    pub name: String,
    pub icon_path: Option<String>,

    pub install_stage: InstanceInstallStage,

    // Main minecraft metadata
    pub game_version: String,
    pub loader: ModLoader,
    pub loader_version: Option<String>,

    // Launch arguments
    pub java_path: Option<String>,
    pub extra_launch_args: Option<Vec<String>>,
    pub custom_env_vars: Option<Vec<(String, String)>>,

    // Minecraft runtime settings
    pub memory: Option<MemorySettings>,
    pub force_fullscreen: Option<bool>,
    pub game_resolution: Option<WindowSize>,

    // Additional information
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub last_played: Option<DateTime<Utc>>,

    pub time_played: u64,
    pub recent_time_played: u64,

    pub hooks: Hooks,

    pub pack_info: Option<PackInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToBytes)]
#[encoding(Json)]
#[serde(rename_all = "camelCase")]
pub struct InstanceFile {
    pub hash: String,
    pub name: Option<String>,
    pub file_name: String,
    pub size: u64,
    pub content_type: ContentType,
    pub path: String,
    pub disabled: bool,
    pub update: Option<HashMap<String, toml::Value>>,
}
