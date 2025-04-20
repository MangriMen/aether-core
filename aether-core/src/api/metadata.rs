use std::time::Duration;

use crate::{
    core::LauncherState,
    features::minecraft::{
        self, CachedMetadataStorage, FsMetadataStorage, ModrinthMetadataStorage,
    },
};

pub async fn get_storage(
) -> crate::Result<minecraft::CachedMetadataStorage<FsMetadataStorage, ModrinthMetadataStorage>> {
    let state = LauncherState::get().await?;

    Ok(CachedMetadataStorage::new(
        FsMetadataStorage::new(&state.locations.cache_dir(), Some(Duration::from_secs(120))),
        ModrinthMetadataStorage::new(state.api_semaphore.clone()),
    ))
}

#[tracing::instrument]
pub async fn get_version_manifest() -> crate::Result<daedalus::minecraft::VersionManifest> {
    minecraft::get_version_manifest(&get_storage().await?).await
}

#[tracing::instrument]
pub async fn get_loader_version_manifest(
    loader: &str,
) -> crate::Result<daedalus::modded::Manifest> {
    minecraft::get_loader_version_manifest(&get_storage().await?, loader).await
}
