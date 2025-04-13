use std::path::PathBuf;

use async_trait::async_trait;

use crate::{
    core::LauncherState,
    features::settings::{Settings, SettingsStorage},
    shared::{read_json_async, write_json_async},
};

pub struct FsSettingsStorage;

impl FsSettingsStorage {
    fn get_settings_file_path(state: &LauncherState) -> PathBuf {
        state.locations.settings_dir.join("settings.json")
    }
}

#[async_trait]
impl SettingsStorage for FsSettingsStorage {
    async fn get(&self, state: &LauncherState) -> crate::Result<Settings> {
        let path = Self::get_settings_file_path(state);
        let settings = read_json_async::<Settings>(path).await?;
        Ok(settings)
    }

    async fn upsert(&self, state: &LauncherState, settings: &Settings) -> crate::Result<()> {
        let path = Self::get_settings_file_path(state);
        write_json_async(path, settings).await?;
        Ok(())
    }
}
