use async_trait::async_trait;

use crate::{core::LauncherState, features::plugins::PluginSettings};

#[async_trait]
pub trait PluginSettingsStorage {
    async fn get(
        &self,
        state: &LauncherState,
        plugin_id: &str,
    ) -> crate::Result<Option<PluginSettings>>;

    async fn upsert(
        &self,
        state: &LauncherState,
        plugin_id: &str,
        settings: &PluginSettings,
    ) -> crate::Result<()>;
}
