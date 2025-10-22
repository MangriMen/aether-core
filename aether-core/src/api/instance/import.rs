use crate::{
    core::domain::LazyLocator,
    features::{
        instance::{ImportInstance, ImportInstanceUseCase, ListImportersUseCase},
        plugins::PluginImporters,
    },
};

pub async fn list_importers() -> crate::Result<Vec<PluginImporters>> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        ListImportersUseCase::new(lazy_locator.get_plugin_registry().await)
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
