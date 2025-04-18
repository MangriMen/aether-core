use std::sync::Arc;

use async_trait::async_trait;
use daedalus::{
    minecraft::VersionManifest,
    modded::{self},
};
use reqwest::Method;

use crate::{
    features::minecraft::ReadMetadataStorage,
    shared::{fetch_json, CachedValue, FetchSemaphore, StorageError},
};

pub const META_URL: &str = "https://launcher-meta.modrinth.com/";

pub struct ModrinthMetadataStorage {
    api_semaphore: Arc<FetchSemaphore>,
}

impl ModrinthMetadataStorage {
    pub fn new(api_semaphore: Arc<FetchSemaphore>) -> Self {
        Self { api_semaphore }
    }

    fn get_loader_manifest_url(loader: &str) -> String {
        format!("{META_URL}{loader}/v0/manifest.json")
    }
}

#[async_trait]
impl ReadMetadataStorage for ModrinthMetadataStorage {
    async fn get_version_manifest(&self) -> Result<CachedValue<VersionManifest>, StorageError> {
        fetch_json::<VersionManifest>(
            Method::GET,
            daedalus::minecraft::VERSION_MANIFEST_URL,
            None,
            None,
            None,
            &self.api_semaphore,
        )
        .await
        .map_err(|err| StorageError::ReadError(err.raw.to_string()))
        .map(CachedValue::new)
    }

    async fn get_loader_version_manifest(
        &self,
        loader: &str,
    ) -> Result<CachedValue<modded::Manifest>, StorageError> {
        fetch_json::<modded::Manifest>(
            Method::GET,
            &Self::get_loader_manifest_url(loader),
            None,
            None,
            None,
            &self.api_semaphore,
        )
        .await
        .map_err(|err| StorageError::ReadError(err.raw.to_string()))
        .map(CachedValue::new)
    }
}
