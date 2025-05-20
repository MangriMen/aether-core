use std::sync::Arc;

use crate::features::minecraft::{ModLoader, ReadMetadataStorage};

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

pub async fn resolve_loader_version(
    game_version: &str,
    loader: &ModLoader,
    loader_version: Option<&str>,
    loader_version_manifest: &daedalus::modded::Manifest,
) -> crate::Result<Option<daedalus::modded::LoaderVersion>> {
    if matches!(loader, ModLoader::Vanilla) {
        return Ok(None);
    }

    if let Some(found_game_version) = loader_version_manifest.game_versions.iter().find(|x| {
        x.id.replace(daedalus::modded::DUMMY_REPLACE_STRING, game_version) == game_version
    }) {
        return Ok(find_loader_version(
            &found_game_version.loaders,
            loader_version.unwrap_or("latest"),
        )
        .cloned());
    }

    Ok(None)
}

fn find_loader_version<'a>(
    loaders: &'a [daedalus::modded::LoaderVersion],
    version: &str,
) -> Option<&'a daedalus::modded::LoaderVersion> {
    match version {
        "latest" => loaders.first(),
        "stable" => loaders
            .iter()
            .find(|x| x.stable)
            .or_else(|| loaders.first()),
        id => loaders.iter().find(|x| x.id == id),
    }
}
