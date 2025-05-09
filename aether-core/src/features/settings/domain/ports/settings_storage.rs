use async_trait::async_trait;

use crate::features::settings::Settings;

#[async_trait]
pub trait SettingsStorage: Send + Sync {
    async fn get(&self) -> crate::Result<Settings>;
    async fn upsert(&self, settings: &Settings) -> crate::Result<()>;
}
