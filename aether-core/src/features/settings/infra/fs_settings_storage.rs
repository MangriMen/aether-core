use std::path::{Path, PathBuf};

use async_trait::async_trait;

use crate::{
    features::settings::{Settings, SettingsStorage},
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

    async fn read(&self) -> crate::Result<Settings> {
        read_json_async(&self.settings_file).await
    }

    async fn write(&self, data: &Settings) -> crate::Result<()> {
        write_json_async(&self.settings_file, &data).await
    }
}

#[async_trait]
impl SettingsStorage for FsSettingsStorage {
    async fn get(&self) -> crate::Result<Settings> {
        self.read()
            .await
            .map_err(|_| crate::ErrorKind::NoValueFor("Settings".to_owned()).as_error())
    }

    async fn upsert(&self, settings: &Settings) -> crate::Result<()> {
        self.write(settings).await
    }
}
