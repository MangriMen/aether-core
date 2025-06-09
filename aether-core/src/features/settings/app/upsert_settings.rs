use std::sync::Arc;

use crate::features::settings::{Settings, SettingsError, SettingsStorage};

pub struct UpsertSettingsUseCase<SS: SettingsStorage> {
    settings_storage: Arc<SS>,
}

impl<SS: SettingsStorage> UpsertSettingsUseCase<SS> {
    pub fn new(settings_storage: Arc<SS>) -> Self {
        Self { settings_storage }
    }

    pub async fn execute(&self, settings: Settings) -> Result<Settings, SettingsError> {
        self.settings_storage.upsert(settings).await
    }
}
