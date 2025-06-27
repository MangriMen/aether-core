use async_trait::async_trait;

use crate::features::settings::{Settings, SettingsError};

#[async_trait]
pub trait SettingsStorage: Send + Sync {
    async fn get(&self) -> Result<Settings, SettingsError>;
    async fn upsert(&self, settings: Settings) -> Result<Settings, SettingsError>;
}
