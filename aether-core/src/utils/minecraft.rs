use std::{collections::HashMap, path::Path};

use daedalus::minecraft::{self, Version};

use crate::{
    launcher::args,
    state::{Instance, Java, LauncherState, MemorySettings},
};

pub fn is_minecraft_updated(
    version_index: usize,
    version_manifest: &minecraft::VersionManifest,
) -> bool {
    version_index
        <= version_manifest
            .versions
            .iter()
            .position(|version| version.id == "22w16a")
            .unwrap_or(0)
}

pub fn get_minecraft_version(
    instance: &Instance,
    version_manifest: minecraft::VersionManifest,
) -> crate::Result<(Version, bool)> {
    let version_index = version_manifest
        .versions
        .iter()
        .position(|version| version.id == instance.game_version)
        .ok_or(crate::ErrorKind::NoValueFor(
            "minecraft versions".to_string(),
        ))?;

    let version = &version_manifest.versions[version_index];

    let minecraft_updated = is_minecraft_updated(version_index, &version_manifest);

    Ok((version.clone(), minecraft_updated))
}

// TODO: Wrap arguments in struct
#[allow(clippy::too_many_arguments)]
pub fn get_minecraft_jvm_arguments(
    state: &LauncherState,
    version_info: &minecraft::VersionInfo,
    natives_dir: &Path,
    client_path: &Path,
    version_jar: String,
    java_version: &Java,
    memory: MemorySettings,
    java_args: &[String],
    args: &HashMap<minecraft::ArgumentType, Vec<minecraft::Argument>>,
    minecraft_updated: bool,
) -> crate::Result<Vec<String>> {
    Ok(args::get_jvm_arguments(
        args.get(&minecraft::ArgumentType::Jvm)
            .map(|x| x.as_slice()),
        natives_dir,
        &state.locations.libraries_dir(),
        &args::get_class_paths(
            &state.locations.libraries_dir(),
            version_info.libraries.as_slice(),
            client_path,
            &java_version.architecture,
            minecraft_updated,
        )?,
        &version_jar,
        memory,
        Vec::from(java_args),
        &java_version.architecture,
    )?
    .into_iter()
    .collect::<Vec<_>>())
}
