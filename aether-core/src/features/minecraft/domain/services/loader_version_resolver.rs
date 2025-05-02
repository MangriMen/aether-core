use std::sync::Arc;

use crate::features::minecraft::{resolve_loader_version, ModLoader, ReadMetadataStorage};

pub struct LoaderVersionResolver<MS> {
    metadata_storage: Arc<MS>,
}

impl<MS: ReadMetadataStorage> LoaderVersionResolver<MS> {
    pub fn new(metadata_storage: Arc<MS>) -> Self {
        Self { metadata_storage }
    }

    pub async fn resolve(
        &self,
        game_version: &str,
        mod_loader: &ModLoader,
        loader_version: &Option<String>,
    ) -> crate::Result<Option<daedalus::modded::LoaderVersion>> {
        if !matches!(mod_loader, ModLoader::Vanilla) {
            let loader_version_manifest = self
                .metadata_storage
                .get_loader_version_manifest(mod_loader.as_meta_str())
                .await?
                .value;

            resolve_loader_version(
                game_version,
                mod_loader,
                loader_version.as_deref(),
                &loader_version_manifest,
            )
            .await
        } else {
            Ok(None)
        }
    }
}
