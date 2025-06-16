use crate::{
    core::domain::LazyLocator,
    features::minecraft::{GetLoaderVersionManifestUseCase, GetVersionManifestUseCase},
};

#[tracing::instrument]
pub async fn get_version_manifest() -> crate::Result<daedalus::minecraft::VersionManifest> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        GetVersionManifestUseCase::new(lazy_locator.get_metadata_storage().await)
            .execute()
            .await?,
    )
}

#[tracing::instrument]
pub async fn get_loader_version_manifest(
    loader: String,
) -> crate::Result<daedalus::modded::Manifest> {
    let lazy_locator = LazyLocator::get().await?;

    Ok(
        GetLoaderVersionManifestUseCase::new(lazy_locator.get_metadata_storage().await)
            .execute(loader)
            .await?,
    )
}
