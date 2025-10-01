use crate::{
    core::domain::LazyLocator,
    features::instance::{
        ImportConfig, ImportInstance, ImportInstanceUseCase, ListImportConfigsUseCase,
    },
};

pub async fn list_import_configs() -> crate::Result<Vec<ImportConfig>> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        ListImportConfigsUseCase::new(lazy_locator.get_plugin_registry().await)
            .execute()
            .await?,
    )
}

pub async fn import(import_instance: ImportInstance) -> crate::Result<()> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        ImportInstanceUseCase::new(lazy_locator.get_plugin_registry().await)
            .execute(import_instance)
            .await?,
    )
}
