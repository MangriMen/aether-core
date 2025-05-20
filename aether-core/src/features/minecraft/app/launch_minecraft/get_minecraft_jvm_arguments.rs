use std::path::Path;

use daedalus::minecraft::{self};

use crate::features::{
    java::Java,
    minecraft::{get_class_paths, get_jvm_arguments},
    settings::MemorySettings,
};

// TODO: Wrap arguments in struct
#[allow(clippy::too_many_arguments)]
pub fn get_minecraft_jvm_arguments(
    arguments: Option<&[minecraft::Argument]>,
    libraries_dir: &Path,
    version_info: &minecraft::VersionInfo,
    natives_dir: &Path,
    client_path: &Path,
    version_jar: String,
    java_version: &Java,
    memory: MemorySettings,
    java_args: &[String],
    minecraft_updated: bool,
) -> crate::Result<Vec<String>> {
    Ok(get_jvm_arguments(
        arguments,
        natives_dir,
        libraries_dir,
        &get_class_paths(
            libraries_dir,
            version_info.libraries.as_slice(),
            client_path,
            &java_version.architecture,
            minecraft_updated,
        )?,
        &version_jar,
        memory,
        java_args,
        &java_version.architecture,
    )?
    .into_iter()
    .collect::<Vec<_>>())
}
