use crate::{
    core::domain::LazyLocator,
    features::settings::{
        EditGlobalInstanceSettings, EditGlobalInstanceSettingsUseCase, EditSettingsUseCase,
        GetGlobalInstanceSettingsUseCase, GetSettingsUseCase, GlobalInstanceSettings, Settings,
    },
};

pub async fn get() -> crate::Result<Settings> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        GetSettingsUseCase::new(lazy_locator.get_settings_storage().await)
            .execute()
            .await?,
    )
}

pub async fn upsert(settings: Settings) -> crate::Result<Settings> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        EditSettingsUseCase::new(lazy_locator.get_settings_storage().await)
            .execute(settings)
            .await?,
    )
}

pub async fn get_global_instance_settings() -> crate::Result<GlobalInstanceSettings> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(GetGlobalInstanceSettingsUseCase::new(
        lazy_locator.get_global_instance_settings_storage().await,
    )
    .execute()
    .await?)
}

pub async fn upsert_global_instance_settings(
    settings: EditGlobalInstanceSettings,
) -> crate::Result<GlobalInstanceSettings> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(EditGlobalInstanceSettingsUseCase::new(
        lazy_locator.get_global_instance_settings_storage().await,
    )
    .execute(settings)
    .await?)
}
