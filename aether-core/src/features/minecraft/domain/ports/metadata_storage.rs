use async_trait::async_trait;

use crate::{features::minecraft::MinecraftError, shared::CachedValue};

#[async_trait]
pub trait ReadMetadataStorage: Send + Sync {
    async fn get_version_manifest(
        &self,
    ) -> Result<CachedValue<daedalus::minecraft::VersionManifest>, MinecraftError>;
    async fn get_loader_version_manifest(
        &self,
        loader: &str,
    ) -> Result<CachedValue<daedalus::modded::Manifest>, MinecraftError>;
}

#[async_trait]
pub trait WriteMetadataStorage: Send + Sync {
    async fn save_version_manifest(
        &self,
        manifest: &daedalus::minecraft::VersionManifest,
    ) -> Result<(), MinecraftError>;
    async fn save_loader_version_manifest(
        &self,
        loader: &str,
        loader_manifest: &daedalus::modded::Manifest,
    ) -> Result<(), MinecraftError>;
}

#[async_trait]
pub trait MetadataStorage: ReadMetadataStorage + WriteMetadataStorage + Send + Sync {}

impl<T> MetadataStorage for T where T: ReadMetadataStorage + WriteMetadataStorage {}
