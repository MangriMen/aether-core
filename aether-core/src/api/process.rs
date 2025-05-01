use uuid::Uuid;

use crate::{
    core::domain::LazyLocator,
    features::process::{
        GetProcessByInstanceIdUseCase, KillProcessUseCase, ListProcessUseCase,
        MinecraftProcessMetadata, WaitForProcessUseCase,
    },
    shared::domain::{AsyncUseCase, AsyncUseCaseWithInput, AsyncUseCaseWithInputAndError},
};

#[tracing::instrument]
pub async fn list() -> crate::Result<Vec<MinecraftProcessMetadata>> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        ListProcessUseCase::new(lazy_locator.get_process_manager().await)
            .execute()
            .await,
    )
}

#[tracing::instrument]
pub async fn get_by_instance_id(id: String) -> crate::Result<Vec<MinecraftProcessMetadata>> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        GetProcessByInstanceIdUseCase::new(lazy_locator.get_process_manager().await)
            .execute(id)
            .await,
    )
}

#[tracing::instrument]
pub async fn kill(uuid: Uuid) -> crate::Result<()> {
    let lazy_locator = LazyLocator::get().await?;

    KillProcessUseCase::new(lazy_locator.get_process_manager().await)
        .execute(uuid)
        .await
}

#[tracing::instrument]
pub async fn wait_for(uuid: Uuid) -> crate::Result<()> {
    let lazy_locator = LazyLocator::get().await?;

    WaitForProcessUseCase::new(lazy_locator.get_process_manager().await)
        .execute(uuid)
        .await
}
