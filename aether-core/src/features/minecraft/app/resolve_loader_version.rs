use crate::features::minecraft::ModLoader;

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
