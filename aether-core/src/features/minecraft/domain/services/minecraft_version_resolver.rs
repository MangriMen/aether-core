pub fn resolve_minecraft_version(
    game_version: &str,
    version_manifest: daedalus::minecraft::VersionManifest,
) -> crate::Result<(daedalus::minecraft::Version, bool)> {
    let (index, version) = version_manifest
        .versions
        .iter()
        .enumerate()
        .find(|(_, v)| v.id == game_version)
        .ok_or_else(|| crate::ErrorKind::NoValueFor("minecraft versions".into()))?;

    let is_updated = is_minecraft_updated(index, &version_manifest);

    Ok((version.clone(), is_updated))
}

fn is_minecraft_updated(
    version_index: usize,
    version_manifest: &daedalus::minecraft::VersionManifest,
) -> bool {
    version_index
        <= version_manifest
            .versions
            .iter()
            .position(|version| version.id == "22w16a")
            .unwrap_or(0)
}
