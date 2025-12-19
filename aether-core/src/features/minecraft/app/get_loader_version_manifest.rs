use std::sync::Arc;

use crate::features::minecraft::{MinecraftDomainError, ModLoader, MetadataStorage};

pub struct GetLoaderVersionManifestUseCase<MS: MetadataStorage> {
    metadata_storage: Arc<MS>,
}

impl<MS: MetadataStorage> GetLoaderVersionManifestUseCase<MS> {
    pub fn new(metadata_storage: Arc<MS>) -> Self {
        Self { metadata_storage }
    }

    pub async fn execute(
        &self,
        loader: ModLoader,
    ) -> Result<daedalus::modded::Manifest, MinecraftDomainError> {
        self.metadata_storage
            .get_loader_version_manifest(loader)
            .await
    }
}
