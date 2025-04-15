use std::path::{Path, PathBuf};

use async_trait::async_trait;

use crate::{
    features::settings::{Settings, SettingsStorage},
    shared::{read_json_async, write_json_async},
};

pub struct FsSettingsStorage {
    path: PathBuf,
}

impl FsSettingsStorage {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
        }
    }
}

#[async_trait]
impl SettingsStorage for FsSettingsStorage {
    async fn get(&self) -> crate::Result<Settings> {
        read_json_async::<Settings>(&self.path).await
    }

    async fn upsert(&self, settings: &Settings) -> crate::Result<()> {
        write_json_async(&self.path, settings).await
    }
}
