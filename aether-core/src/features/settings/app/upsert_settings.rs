use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::settings::{Settings, SettingsStorage},
    shared::AsyncUseCaseWithInputAndError,
};

pub async fn upsert_settings<S>(storage: &S, settings: &Settings) -> crate::Result<()>
where
    S: SettingsStorage + ?Sized,
{
    storage.upsert(settings).await
}

pub struct UpsertSettingsUseCase<SS: SettingsStorage> {
    settings_storage: Arc<SS>,
}

impl<SS: SettingsStorage> UpsertSettingsUseCase<SS> {
    pub fn new(settings_storage: Arc<SS>) -> Self {
        Self { settings_storage }
    }
}

#[async_trait]
impl<SS: SettingsStorage> AsyncUseCaseWithInputAndError for UpsertSettingsUseCase<SS> {
    type Input = Settings;
    type Output = ();
    type Error = crate::Error;

    async fn execute(&self, settings: Self::Input) -> Result<Self::Output, Self::Error> {
        self.settings_storage.upsert(&settings).await
    }
}
