use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::settings::{Settings, SettingsStorage},
    shared::domain::AsyncUseCase,
};

pub async fn upsert_settings<S>(storage: &S, settings: &Settings) -> crate::Result<()>
where
    S: SettingsStorage + ?Sized,
{
    storage.upsert(settings).await
}

pub struct UpsertSettingsUseCase<SS: SettingsStorage> {
    storage: Arc<SS>,
}

impl<SS: SettingsStorage> UpsertSettingsUseCase<SS> {
    pub fn new(storage: Arc<SS>) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl<SS> AsyncUseCase for UpsertSettingsUseCase<SS>
where
    SS: SettingsStorage + Send + Sync,
{
    type Input = Settings;
    type Output = ();
    type Error = crate::Error;

    async fn execute(&self, settings: Self::Input) -> Result<Self::Output, Self::Error> {
        self.storage.upsert(&settings).await
    }
}
