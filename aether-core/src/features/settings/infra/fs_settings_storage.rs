use std::path::Path;

use async_trait::async_trait;

use crate::{
    features::settings::{Settings, SettingsStorage},
    shared::infra::{AsyncFileDb, AsyncJsonDb},
};

pub struct FsSettingsStorage {
    db: AsyncJsonDb<Settings>,
}

impl FsSettingsStorage {
    pub fn new(path: &Path) -> Self {
        Self {
            db: AsyncJsonDb::new(path.to_path_buf()),
        }
    }
}

#[async_trait]
impl SettingsStorage for FsSettingsStorage {
    async fn get(&self) -> crate::Result<Settings> {
        self.db
            .read()
            .await?
            .ok_or_else(|| crate::ErrorKind::NoValueFor("Settings".to_owned()).as_error())
    }

    async fn upsert(&self, settings: &Settings) -> crate::Result<()> {
        self.db.write(settings).await
    }
}
