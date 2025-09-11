use async_trait::async_trait;

use crate::features::settings::{DefaultInstanceSettings, SettingsError};

#[async_trait]
pub trait DefaultInstanceSettingsStorage: Send + Sync {
    async fn get(&self) -> Result<DefaultInstanceSettings, SettingsError>;
    async fn upsert(
        &self,
        settings: DefaultInstanceSettings,
    ) -> Result<DefaultInstanceSettings, SettingsError>;
}
