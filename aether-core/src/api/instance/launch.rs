use crate::{
    core::domain::LazyLocator,
    features::{
        auth::Credentials,
        minecraft::{self},
        process::MinecraftProcessMetadata,
    },
};

#[tracing::instrument]
pub async fn run(instance_id: &str) -> crate::Result<MinecraftProcessMetadata> {
    let lazy_locator = LazyLocator::get().await?;

    minecraft::run(
        &*lazy_locator.get_settings_storage().await,
        &*lazy_locator.get_auth_storage().await,
        &*lazy_locator.get_instance_manager().await,
        lazy_locator.get_metadata_storage().await,
        instance_id,
    )
    .await
}

#[tracing::instrument]
pub async fn run_credentials(
    id: &str,
    credentials: &Credentials,
) -> crate::Result<MinecraftProcessMetadata> {
    let lazy_locator = LazyLocator::get().await?;

    minecraft::run_credentials(
        &*lazy_locator.get_settings_storage().await,
        &*lazy_locator.get_instance_manager().await,
        lazy_locator.get_metadata_storage().await,
        id,
        credentials,
    )
    .await
}
