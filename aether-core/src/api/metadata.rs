use daedalus::{minecraft, modded};

use crate::{
    core::LauncherState,
    features::launcher::{download_loaders_manifests, download_version_manifest},
};

#[tracing::instrument]
pub async fn get_versions_manifest() -> crate::Result<minecraft::VersionManifest> {
    let state = LauncherState::get().await?;
    download_version_manifest(&state, true).await
}

#[tracing::instrument]
pub async fn get_loader_versions(loader: &str) -> crate::Result<modded::Manifest> {
    let state = LauncherState::get().await?;
    download_loaders_manifests(&state, loader, true)
        .await
        .map_err(|_| crate::ErrorKind::NoValueFor(format!("{} loader versions", loader)).as_error())
}
