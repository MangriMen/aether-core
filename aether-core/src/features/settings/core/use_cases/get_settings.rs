use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::settings::{Settings, SettingsStorage},
    shared::domain::AsyncUseCaseWithError,
};

pub struct GetSettingsUseCase<SS: SettingsStorage> {
    storage: Arc<SS>,
}

impl<SS: SettingsStorage> GetSettingsUseCase<SS> {
    pub fn new(storage: Arc<SS>) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl<SS> AsyncUseCaseWithError for GetSettingsUseCase<SS>
where
    SS: SettingsStorage + Send + Sync,
{
    type Output = Settings;
    type Error = crate::Error;

    async fn execute(&self) -> Result<Self::Output, Self::Error> {
        self.storage.get().await
    }
}
