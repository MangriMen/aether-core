use uuid::Uuid;

use crate::{
    core::domain::LazyLocator,
    features::process::{
        GetProcessMetadataByInstanceIdUseCase, KillProcessUseCase, ListProcessMetadataUseCase,
        MinecraftProcessMetadata, WaitForProcessUseCase,
    },
};

#[tracing::instrument]
pub async fn list() -> crate::Result<Vec<MinecraftProcessMetadata>> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        ListProcessMetadataUseCase::new(lazy_locator.get_process_storage().await)
            .execute()
            .await,
    )
}

#[tracing::instrument]
pub async fn get_by_instance_id(
    instance_id: String,
) -> crate::Result<Vec<MinecraftProcessMetadata>> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        GetProcessMetadataByInstanceIdUseCase::new(lazy_locator.get_process_storage().await)
            .execute(instance_id)
            .await,
    )
}

#[tracing::instrument]
pub async fn kill(uuid: Uuid) -> crate::Result<()> {
    let lazy_locator = LazyLocator::get().await?;

    KillProcessUseCase::new(lazy_locator.get_process_storage().await)
        .execute(uuid)
        .await
}

#[tracing::instrument]
pub async fn wait_for(uuid: Uuid) -> crate::Result<()> {
    let lazy_locator = LazyLocator::get().await?;

    WaitForProcessUseCase::new(lazy_locator.get_process_storage().await)
        .execute(uuid)
        .await
}
