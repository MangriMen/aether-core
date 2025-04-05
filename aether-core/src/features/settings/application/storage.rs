use async_trait::async_trait;

use crate::{core::LauncherState, features::settings::Settings};

#[async_trait]
pub trait SettingsStorage {
    async fn get(&self, state: &LauncherState) -> crate::Result<Settings>;
    async fn upsert(&self, state: &LauncherState, settings: &Settings) -> crate::Result<()>;
}
