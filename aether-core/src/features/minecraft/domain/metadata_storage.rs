use async_trait::async_trait;

use crate::shared::{infra::StorageError, CachedValue};

#[async_trait]
pub trait ReadMetadataStorage {
    async fn get_version_manifest(
        &self,
    ) -> Result<CachedValue<daedalus::minecraft::VersionManifest>, StorageError>;
    async fn get_loader_version_manifest(
        &self,
        loader: &str,
    ) -> Result<CachedValue<daedalus::modded::Manifest>, StorageError>;
}

#[async_trait]
pub trait WriteMetadataStorage {
    async fn save_version_manifest(
        &self,
        manifest: &daedalus::minecraft::VersionManifest,
    ) -> Result<(), StorageError>;
    async fn save_loader_version_manifest(
        &self,
        loader: &str,
        loader_manifest: &daedalus::modded::Manifest,
    ) -> Result<(), StorageError>;
}

#[async_trait]
pub trait MetadataStorage: ReadMetadataStorage + WriteMetadataStorage {}

impl<T> MetadataStorage for T where T: ReadMetadataStorage + WriteMetadataStorage {}
