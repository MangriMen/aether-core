use crate::{
    core::domain::LazyLocator,
    features::instance::{ImportInstance, ImportInstanceUseCase},
};

pub async fn import(import_instance: ImportInstance) -> crate::Result<()> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        ImportInstanceUseCase::new(lazy_locator.get_plugin_registry().await)
            .execute(import_instance)
            .await?,
    )
}
