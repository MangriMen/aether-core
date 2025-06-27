use std::sync::Arc;

use crate::features::minecraft::{MinecraftError, ReadMetadataStorage};

pub struct GetVersionManifestUseCase<MS: ReadMetadataStorage> {
    metadata_storage: Arc<MS>,
}

impl<MS: ReadMetadataStorage> GetVersionManifestUseCase<MS> {
    pub fn new(metadata_storage: Arc<MS>) -> Self {
        Self { metadata_storage }
    }

    pub async fn execute(&self) -> Result<daedalus::minecraft::VersionManifest, MinecraftError> {
        Ok(self.metadata_storage.get_version_manifest().await?.value)
    }
}
