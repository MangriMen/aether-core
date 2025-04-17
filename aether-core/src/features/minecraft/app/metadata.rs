use daedalus::{minecraft, modded};

use crate::features::minecraft::ReadMetadataStorage;

pub async fn get_version_manifest<S>(storage: &S) -> crate::Result<minecraft::VersionManifest>
where
    S: ReadMetadataStorage + ?Sized,
{
    Ok(storage.get_version_manifest().await?.value)
}

pub async fn get_loader_version_manifest<S>(
    storage: &S,
    loader: &str,
) -> crate::Result<modded::Manifest>
where
    S: ReadMetadataStorage + ?Sized,
{
    Ok(storage.get_loader_version_manifest(loader).await?.value)
}
