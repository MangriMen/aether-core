use std::path::{Path, PathBuf};

use async_trait::async_trait;
use daedalus::{
    minecraft::VersionManifest,
    modded::{self},
};
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    features::minecraft::{ReadMetadataStorage, WriteMetadataStorage},
    shared::{domain::Cacheable, read_json_async, write_json_async, CachedValue, StorageError},
};

pub struct FsMetadataStorage {
    cache_dir: PathBuf,
    ttl: Option<std::time::Duration>,
}

impl FsMetadataStorage {
    pub fn new(cache_dir: &Path, ttl: Option<std::time::Duration>) -> Self {
        Self {
            cache_dir: cache_dir.to_path_buf(),
            ttl,
        }
    }

    pub async fn read<T>(&self, path: &Path) -> Result<CachedValue<T>, StorageError>
    where
        T: DeserializeOwned,
    {
        if !path.exists() {
            return Err(StorageError::NotFound {
                path: path.to_path_buf(),
            });
        }

        let value = read_json_async::<CachedValue<T>>(path)
            .await
            .map_err(|err| StorageError::ReadError(err.to_string()))?;

        if let Some(ttl) = self.ttl {
            if value.is_expired(ttl) {
                return Err(StorageError::CacheExpired {
                    path: path.to_path_buf(),
                });
            }
        }

        Ok(value)
    }

    pub async fn write<T>(&self, path: &Path, value: &T) -> Result<(), StorageError>
    where
        T: Serialize,
    {
        write_json_async(path, CachedValue::new(value))
            .await
            .map_err(|err| StorageError::WriteError(err.to_string()))
    }

    fn version_manifest_path(&self) -> PathBuf {
        self.cache_dir.join("version_manifest.json")
    }

    fn loader_manifest_path(&self, loader: &str) -> PathBuf {
        self.cache_dir
            .join("mod_loaders")
            .join(format!("{loader}-manifest.json"))
    }
}

#[async_trait]
impl ReadMetadataStorage for FsMetadataStorage {
    async fn get_version_manifest(&self) -> Result<CachedValue<VersionManifest>, StorageError> {
        self.read(&self.version_manifest_path()).await
    }

    async fn get_loader_version_manifest(
        &self,
        loader: &str,
    ) -> Result<CachedValue<modded::Manifest>, StorageError> {
        self.read(&self.loader_manifest_path(loader)).await
    }
}

#[async_trait]
impl WriteMetadataStorage for FsMetadataStorage {
    async fn save_version_manifest(&self, manifest: &VersionManifest) -> Result<(), StorageError> {
        self.write(&self.version_manifest_path(), manifest).await
    }

    async fn save_loader_version_manifest(
        &self,
        loader: &str,
        loader_manifest: &modded::Manifest,
    ) -> Result<(), StorageError> {
        self.write(&self.loader_manifest_path(loader), loader_manifest)
            .await
    }
}
