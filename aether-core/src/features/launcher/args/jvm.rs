use std::{collections::HashSet, path::Path};

use daedalus::minecraft;

use crate::{
    features::{launcher::parse_rules, settings::MemorySettings},
    shared::canonicalize,
    shared::utils::get_classpath_separator,
};

use super::parse_arguments;

// TODO: Wrap arguments in struct
#[allow(clippy::too_many_arguments)]
pub fn get_jvm_arguments(
    arguments: Option<&[minecraft::Argument]>,
    natives_path: &Path,
    libraries_path: &Path,
    class_paths: &str,
    version_name: &str,
    memory: MemorySettings,
    custom_args: Vec<String>,
    java_arch: &str,
) -> crate::Result<Vec<String>> {
    let mut parsed_arguments = Vec::new();

    if let Some(args) = arguments {
        parse_arguments(
            args,
            &mut parsed_arguments,
            |arg| {
                parse_jvm_argument(
                    arg.to_string(),
                    natives_path,
                    libraries_path,
                    class_paths,
                    version_name,
                    java_arch,
                )
            },
            java_arch,
        )?;
    } else {
        parsed_arguments.push(format!(
            "-Djava.library.path={}",
            canonicalize(natives_path)
                .map_err(|_| crate::ErrorKind::LauncherError(format!(
                    "Specified natives path {} does not exist",
                    natives_path.to_string_lossy()
                ))
                .as_error())?
                .to_string_lossy()
        ));
        parsed_arguments.push("-cp".to_string());
        parsed_arguments.push(class_paths.to_string());
    }
    parsed_arguments.push(format!("-Xmx{}M", memory.maximum));
    for arg in custom_args {
        if !arg.is_empty() {
            parsed_arguments.push(arg);
        }
    }

    Ok(parsed_arguments)
}

fn parse_jvm_argument(
    mut argument: String,
    natives_path: &Path,
    libraries_path: &Path,
    class_paths: &str,
    version_name: &str,
    java_arch: &str,
) -> crate::Result<String> {
    argument.retain(|c| !c.is_whitespace());
    Ok(argument
        .replace(
            "${natives_directory}",
            &canonicalize(natives_path)
                .map_err(|_| {
                    crate::ErrorKind::LauncherError(format!(
                        "Specified natives path {} does not exist",
                        natives_path.to_string_lossy()
                    ))
                    .as_error()
                })?
                .to_string_lossy(),
        )
        .replace(
            "${library_directory}",
            &canonicalize(libraries_path)
                .map_err(|_| {
                    crate::ErrorKind::LauncherError(format!(
                        "Specified libraries path {} does not exist",
                        libraries_path.to_string_lossy()
                    ))
                    .as_error()
                })?
                .to_string_lossy(),
        )
        .replace("${classpath_separator}", get_classpath_separator(java_arch))
        .replace("${launcher_name}", "theseus")
        .replace("${launcher_version}", env!("CARGO_PKG_VERSION"))
        .replace("${version_name}", version_name)
        .replace("${classpath}", class_paths))
}

pub fn get_class_paths(
    libraries_path: &Path,
    libraries: &[minecraft::Library],
    client_path: &Path,
    java_arch: &str,
    minecraft_updated: bool,
) -> crate::Result<String> {
    let mut cps = libraries
        .iter()
        .filter_map(|library| {
            if let Some(rules) = &library.rules {
                if !parse_rules(rules, java_arch, minecraft_updated) {
                    return None;
                }
            }

            if !library.include_in_classpath {
                return None;
            }

            Some(get_lib_path(libraries_path, &library.name, false))
        })
        .collect::<Result<HashSet<_>, _>>()?;

    cps.insert(
        canonicalize(client_path)
            .map_err(|_| {
                crate::ErrorKind::LauncherError(format!(
                    "Specified class path {} does not exist",
                    client_path.to_string_lossy()
                ))
                .as_error()
            })?
            .to_string_lossy()
            .to_string(),
    );

    Ok(cps
        .into_iter()
        .collect::<Vec<_>>()
        .join(get_classpath_separator(java_arch)))
}

pub fn get_class_paths_jar<T: AsRef<str>>(
    libraries_path: &Path,
    libraries: &[T],
    java_arch: &str,
) -> crate::Result<String> {
    let cps = libraries
        .iter()
        .map(|library| get_lib_path(libraries_path, library.as_ref(), false))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(cps.join(get_classpath_separator(java_arch)))
}

pub fn get_lib_path(
    libraries_path: &Path,
    lib: &str,
    allow_not_exist: bool,
) -> crate::Result<String> {
    let mut path = libraries_path.to_path_buf();

    path.push(daedalus::get_path_from_artifact(lib)?);

    if !path.exists() && allow_not_exist {
        return Ok(path.to_string_lossy().to_string());
    }

    let path = &canonicalize(&path).map_err(|_| {
        crate::ErrorKind::LauncherError(format!(
            "Library file at path {} does not exist",
            path.to_string_lossy()
        ))
        .as_error()
    })?;

    Ok(path.to_string_lossy().to_string())
}
