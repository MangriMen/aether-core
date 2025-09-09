use std::path::{Path, PathBuf};

use async_trait::async_trait;

use crate::{
    features::settings::{GlobalInstanceSettings, GlobalInstanceSettingsStorage, SettingsError},
    shared::{ensure_read_json_async, write_json_async},
};

pub struct FsGlobalInstanceSettingsStorage {
    settings_file: PathBuf,
}

impl FsGlobalInstanceSettingsStorage {
    pub fn new(settings_dir: &Path) -> Self {
        Self {
            settings_file: settings_dir.join("instance_settings.json"),
        }
    }
}

#[async_trait]
impl GlobalInstanceSettingsStorage for FsGlobalInstanceSettingsStorage {
    async fn get(&self) -> Result<GlobalInstanceSettings, SettingsError> {
        Ok(ensure_read_json_async(&self.settings_file).await?)
    }

    async fn upsert(
        &self,
        settings: GlobalInstanceSettings,
    ) -> Result<GlobalInstanceSettings, SettingsError> {
        write_json_async(&self.settings_file, &settings).await?;
        Ok(settings)
    }
}
