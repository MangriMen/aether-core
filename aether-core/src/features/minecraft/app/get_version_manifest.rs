use std::sync::Arc;

use crate::features::minecraft::{MinecraftDomainError, MetadataStorage};

pub struct GetVersionManifestUseCase<MS: MetadataStorage> {
    metadata_storage: Arc<MS>,
}

impl<MS: MetadataStorage> GetVersionManifestUseCase<MS> {
    pub fn new(metadata_storage: Arc<MS>) -> Self {
        Self { metadata_storage }
    }

    pub async fn execute(&self) -> Result<daedalus::minecraft::VersionManifest, MinecraftDomainError> {
        self.metadata_storage.get_version_manifest().await
    }
}
