use crate::{
    core::domain::LazyLocator,
    features::settings::{GetSettingsUseCase, Settings, UpsertSettingsUseCase},
};

pub async fn get() -> crate::Result<Settings> {
    let lazy_locator = LazyLocator::get().await?;

    GetSettingsUseCase::new(lazy_locator.get_settings_storage().await)
        .execute()
        .await
}

pub async fn upsert(settings: Settings) -> crate::Result<()> {
    let lazy_locator = LazyLocator::get().await?;

    UpsertSettingsUseCase::new(lazy_locator.get_settings_storage().await)
        .execute(settings)
        .await
}
