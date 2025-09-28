use std::path::{Path, PathBuf};

use async_trait::async_trait;

use crate::{
    features::settings::{DefaultInstanceSettings, DefaultInstanceSettingsStorage, SettingsError},
    shared::{ensure_read_json_async, write_json_async},
};

pub struct FsDefaultInstanceSettingsStorage {
    settings_file: PathBuf,
}

impl FsDefaultInstanceSettingsStorage {
    pub fn new(settings_dir: &Path) -> Self {
        Self {
            settings_file: settings_dir.join("instance_settings.json"),
        }
    }
}

#[async_trait]
impl DefaultInstanceSettingsStorage for FsDefaultInstanceSettingsStorage {
    async fn get(&self) -> Result<DefaultInstanceSettings, SettingsError> {
        Ok(ensure_read_json_async(&self.settings_file).await?)
    }

    async fn upsert(
        &self,
        settings: DefaultInstanceSettings,
    ) -> Result<DefaultInstanceSettings, SettingsError> {
        write_json_async(&self.settings_file, &settings).await?;
        Ok(settings)
    }
}
