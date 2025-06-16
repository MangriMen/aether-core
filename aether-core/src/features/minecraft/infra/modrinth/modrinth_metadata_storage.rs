use std::sync::Arc;

use async_trait::async_trait;
use daedalus::{
    minecraft::VersionManifest,
    modded::{self},
};

use crate::{
    features::minecraft::{MinecraftError, ReadMetadataStorage},
    libs::request_client::{Request, RequestClient, RequestClientExt},
    shared::{CachedValue, IoError},
};

pub const META_URL: &str = "https://launcher-meta.modrinth.com/";

pub struct ModrinthMetadataStorage<RC: RequestClient> {
    request_client: Arc<RC>,
}

impl<RC: RequestClient> ModrinthMetadataStorage<RC> {
    pub fn new(request_client: Arc<RC>) -> Self {
        Self { request_client }
    }

    fn get_loader_manifest_url(loader: &str) -> String {
        format!("{META_URL}{loader}/v0/manifest.json")
    }
}

#[async_trait]
impl<RC: RequestClient> ReadMetadataStorage for ModrinthMetadataStorage<RC> {
    async fn get_version_manifest(&self) -> Result<CachedValue<VersionManifest>, MinecraftError> {
        Ok(self
            .request_client
            .fetch_json_with_progress(
                Request::get(daedalus::minecraft::VERSION_MANIFEST_URL),
                None,
            )
            .await
            .map_err(|err| {
                IoError::IOError(std::io::Error::new(
                    std::io::ErrorKind::NetworkUnreachable,
                    err,
                ))
            })
            .map(CachedValue::new)?)
    }

    async fn get_loader_version_manifest(
        &self,
        loader: &str,
    ) -> Result<CachedValue<modded::Manifest>, MinecraftError> {
        Ok(self
            .request_client
            .fetch_json_with_progress(Request::get(Self::get_loader_manifest_url(loader)), None)
            .await
            .map_err(|err| {
                IoError::IOError(std::io::Error::new(
                    std::io::ErrorKind::NetworkUnreachable,
                    err,
                ))
            })
            .map(CachedValue::new)?)
    }
}
