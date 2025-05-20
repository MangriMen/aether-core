use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::settings::{Settings, SettingsStorage},
    shared::AsyncUseCaseWithError,
};

pub struct GetSettingsUseCase<SS: SettingsStorage> {
    settings_storage: Arc<SS>,
}

impl<SS: SettingsStorage> GetSettingsUseCase<SS> {
    pub fn new(settings_storage: Arc<SS>) -> Self {
        Self { settings_storage }
    }
}

#[async_trait]
impl<SS: SettingsStorage> AsyncUseCaseWithError for GetSettingsUseCase<SS> {
    type Output = Settings;
    type Error = crate::Error;

    async fn execute(&self) -> Result<Self::Output, Self::Error> {
        self.settings_storage.get().await
    }
}
