use std::path::{Path, PathBuf};

use async_trait::async_trait;

use crate::{
    features::settings::{Settings, SettingsError, SettingsStorage},
    shared::{read_json_async, write_json_async},
};

pub struct FsSettingsStorage {
    settings_file: PathBuf,
}

impl FsSettingsStorage {
    pub fn new(settings_dir: &Path) -> Self {
        Self {
            settings_file: settings_dir.join("settings.json"),
        }
    }

    async fn read(&self) -> Result<Settings, SettingsError> {
        Ok(read_json_async(&self.settings_file).await?)
    }

    async fn write(&self, data: &Settings) -> Result<(), SettingsError> {
        Ok(write_json_async(&self.settings_file, &data).await?)
    }
}

#[async_trait]
impl SettingsStorage for FsSettingsStorage {
    async fn get(&self) -> Result<Settings, SettingsError> {
        self.read().await
    }

    async fn upsert(&self, settings: Settings) -> Result<Settings, SettingsError> {
        self.write(&settings).await?;
        Ok(settings)
    }
}
