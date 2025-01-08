use crate::utils::io::{read_json_async, write_json_async};

use super::{Hooks, LauncherState};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Settings {
    pub launcher_dir: Option<String>,
    pub metadata_dir: Option<String>,
    pub plugins_dir: Option<String>,

    pub max_concurrent_downloads: usize,

    pub memory: MemorySettings,

    pub game_resolution: WindowSize,
    pub custom_env_vars: Vec<(String, String)>,
    pub extra_launch_args: Vec<String>,

    pub hooks: Hooks,
}

impl Settings {
    pub async fn get(state: &LauncherState) -> crate::Result<Settings> {
        let path = state.locations.settings_dir.join("settings.json");
        read_json_async::<Settings>(path).await
    }

    pub async fn update(state: &LauncherState, settings: &Settings) -> crate::Result<()> {
        let path = state.locations.settings_dir.join("settings.json");
        write_json_async(&path, settings).await
    }
}

/// Minecraft memory settings
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
pub struct MemorySettings {
    pub maximum: u32,
}

/// Game window size
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
pub struct WindowSize(pub u16, pub u16);
