use async_trait::async_trait;

use crate::features::settings::{GlobalInstanceSettings, SettingsError};

#[async_trait]
pub trait GlobalInstanceSettingsStorage: Send + Sync {
    async fn get(&self) -> Result<GlobalInstanceSettings, SettingsError>;
    async fn upsert(
        &self,
        settings: GlobalInstanceSettings,
    ) -> Result<GlobalInstanceSettings, SettingsError>;
}
