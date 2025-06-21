use std::sync::Arc;

use crate::features::minecraft::{MinecraftError, ModLoader, ReadMetadataStorage};

pub struct GetLoaderVersionManifestUseCase<MS: ReadMetadataStorage> {
    metadata_storage: Arc<MS>,
}

impl<MS: ReadMetadataStorage> GetLoaderVersionManifestUseCase<MS> {
    pub fn new(metadata_storage: Arc<MS>) -> Self {
        Self { metadata_storage }
    }

    pub async fn execute(
        &self,
        loader: ModLoader,
    ) -> Result<daedalus::modded::Manifest, MinecraftError> {
        Ok(self
            .metadata_storage
            .get_loader_version_manifest(loader)
            .await?
            .value)
    }
}
