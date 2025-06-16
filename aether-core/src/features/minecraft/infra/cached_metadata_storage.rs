use async_trait::async_trait;
use daedalus::{minecraft::VersionManifest, modded};

use crate::{
    features::minecraft::{MetadataStorage, MinecraftError, ReadMetadataStorage},
    shared::CachedValue,
};

pub struct CachedMetadataStorage<L: MetadataStorage, R: ReadMetadataStorage> {
    local_storage: L,
    remote_storage: R,
}

impl<L: MetadataStorage, R: ReadMetadataStorage> CachedMetadataStorage<L, R> {
    pub fn new(local_storage: L, remote_storage: R) -> Self {
        Self {
            local_storage,
            remote_storage,
        }
    }
}

#[async_trait]
impl<L: MetadataStorage, R: ReadMetadataStorage> ReadMetadataStorage
    for CachedMetadataStorage<L, R>
{
    async fn get_version_manifest(&self) -> Result<CachedValue<VersionManifest>, MinecraftError> {
        let local_manifest = self.local_storage.get_version_manifest().await;

        if let Ok(local_manifest) = local_manifest {
            return Ok(local_manifest);
        }

        match self.remote_storage.get_version_manifest().await {
            Ok(remote_manifest) => {
                self.local_storage
                    .save_version_manifest(&remote_manifest.value)
                    .await?;
                Ok(remote_manifest)
            }
            Err(_) => local_manifest,
        }
    }

    async fn get_loader_version_manifest(
        &self,
        loader: &str,
    ) -> Result<CachedValue<modded::Manifest>, MinecraftError> {
        let local_loader_manifest = self.local_storage.get_loader_version_manifest(loader).await;

        if let Ok(local_loader_manifest) = local_loader_manifest {
            return Ok(local_loader_manifest);
        }

        match self
            .remote_storage
            .get_loader_version_manifest(loader)
            .await
        {
            Ok(remote_loader_manifest) => {
                self.local_storage
                    .save_loader_version_manifest(loader, &remote_loader_manifest.value)
                    .await?;
                Ok(remote_loader_manifest)
            }
            Err(_) => local_loader_manifest,
        }
    }
}
