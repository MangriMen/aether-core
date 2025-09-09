use std::sync::Arc;

use crate::features::settings::{
    GlobalInstanceSettings, GlobalInstanceSettingsStorage, SettingsError,
};

pub struct GetGlobalInstanceSettingsUseCase<SS: GlobalInstanceSettingsStorage> {
    instance_settings_storage: Arc<SS>,
}

impl<SS: GlobalInstanceSettingsStorage> GetGlobalInstanceSettingsUseCase<SS> {
    pub fn new(instance_settings_storage: Arc<SS>) -> Self {
        Self {
            instance_settings_storage,
        }
    }

    pub async fn execute(&self) -> Result<GlobalInstanceSettings, SettingsError> {
        self.instance_settings_storage.get().await
    }
}
