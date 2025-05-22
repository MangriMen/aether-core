use std::sync::Arc;

use crate::features::minecraft::ReadMetadataStorage;

pub struct GetLoaderVersionManifestUseCase<MS: ReadMetadataStorage> {
    metadata_storage: Arc<MS>,
}

impl<MS: ReadMetadataStorage> GetLoaderVersionManifestUseCase<MS> {
    pub fn new(metadata_storage: Arc<MS>) -> Self {
        Self { metadata_storage }
    }

    pub async fn execute(&self, loader: String) -> crate::Result<daedalus::modded::Manifest> {
        Ok(self
            .metadata_storage
            .get_loader_version_manifest(&loader)
            .await?
            .value)
    }
}
