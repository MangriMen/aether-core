use std::sync::Arc;

use crate::features::minecraft::ReadMetadataStorage;

pub struct GetVersionManifestUseCase<MS: ReadMetadataStorage> {
    metadata_storage: Arc<MS>,
}

impl<MS: ReadMetadataStorage> GetVersionManifestUseCase<MS> {
    pub fn new(metadata_storage: Arc<MS>) -> Self {
        Self { metadata_storage }
    }

    pub async fn execute(&self) -> crate::Result<daedalus::minecraft::VersionManifest> {
        Ok(self.metadata_storage.get_version_manifest().await?.value)
    }
}
