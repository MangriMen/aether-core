use serde::{Deserialize, Serialize};

use crate::features::settings::{Hooks, MemorySettings, WindowSize};

#[derive(Serialize, Deserialize, Debug)]
pub struct LaunchSettings {
    pub extra_launch_args: Vec<String>,
    pub custom_env_vars: Vec<(String, String)>,
    pub memory: MemorySettings,
    pub game_resolution: WindowSize,
    pub hooks: Hooks,
}
