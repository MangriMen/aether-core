use std::sync::Arc;

use crate::features::settings::{Settings, SettingsStorage};

pub struct GetSettingsUseCase<SS: SettingsStorage> {
    settings_storage: Arc<SS>,
}

impl<SS: SettingsStorage> GetSettingsUseCase<SS> {
    pub fn new(settings_storage: Arc<SS>) -> Self {
        Self { settings_storage }
    }

    pub async fn execute(&self) -> crate::Result<Settings> {
        self.settings_storage.get().await
    }
}
