use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use daedalus::{
    minecraft::VersionInfo,
    modded::{self},
};
use tokio::process::Command;

use crate::{
    core::LauncherState,
    features::{
        events::{emit_loading, LoadingBarId},
        instance::Instance,
        java::Java,
        minecraft,
    },
    processor_rules,
    shared::IOError,
    with_mut_ref,
};

#[tracing::instrument]
pub async fn run_forge_processors(
    instance: &Instance,
    version_jar: String,
    instance_path: &PathBuf,
    version_info: &mut VersionInfo,
    java_version: &Java,
    loading_bar: Option<&LoadingBarId>,
) -> crate::Result<()> {
    let state = LauncherState::get().await?;

    if let Some(processors) = &version_info.processors {
        let client_path = state
            .locations
            .version_dir(&version_jar)
            .join(format!("{version_jar}.jar"));

        let libraries_dir = state.locations.libraries_dir();

        if let Some(ref mut data) = version_info.data {
            processor_rules! {
                data;
                "SIDE":
                    client => "client",
                    server => "";
                "MINECRAFT_JAR" :
                    client => client_path.to_string_lossy(),
                    server => "";
                "MINECRAFT_VERSION":
                    client => instance.game_version.clone(),
                    server => "";
                "ROOT":
                    client => instance_path.to_string_lossy(),
                    server => "";
                "LIBRARY_DIR":
                    client => libraries_dir.to_string_lossy(),
                    server => "";
            }

            if let Some(loading_bar) = loading_bar {
                emit_loading(loading_bar, 0.0, Some("Running forge processors")).await?;
            };

            let total_length = processors.len();

            // Forge processors (90-100)
            for (index, processor) in processors.iter().enumerate() {
                if let Some(sides) = &processor.sides {
                    if !sides.contains(&String::from("client")) {
                        continue;
                    }
                }

                run_forge_processor(processor, data, &libraries_dir, java_version).await?;

                if let Some(loading_bar) = loading_bar {
                    emit_loading(
                        loading_bar,
                        30.0 / total_length as f64,
                        Some(&format!(
                            "Running forge processor {}/{}",
                            index, total_length
                        )),
                    )
                    .await?;
                }
            }
        }
    }

    Ok(())
}

pub async fn run_forge_processor(
    processor: &modded::Processor,
    data: &HashMap<String, modded::SidedDataEntry>,
    libraries_dir: &Path,
    java_version: &Java,
) -> crate::Result<()> {
    log::debug!("Running forge processor {}", processor.jar);

    let class_path: Vec<String> = with_mut_ref!(cp = processor.classpath.clone() => {
        cp.push(processor.jar.clone())
    });

    let class_path_arg =
        minecraft::get_class_paths_jar(libraries_dir, &class_path, &java_version.architecture)?;

    let processor_main_class = get_processor_main_class(minecraft::get_lib_path(
        libraries_dir,
        &processor.jar,
        false,
    )?)
    .await?
    .ok_or_else(|| {
        crate::ErrorKind::LauncherError(format!(
            "Could not find processor main class for {}",
            processor.jar
        ))
    })?;

    let processor_args = get_processor_arguments(libraries_dir, &processor.args, data)?;

    let child = Command::new(&java_version.path)
        .arg("-cp")
        .arg(class_path_arg)
        .arg(processor_main_class)
        .args(processor_args)
        .output()
        .await
        .map_err(|e| IOError::with_path(e, &java_version.path))
        .map_err(|err| {
            crate::ErrorKind::LauncherError(format!("Error running processor: {err}",))
        })?;

    if !child.status.success() {
        return Err(crate::ErrorKind::LauncherError(format!(
            "Processor error: {}",
            String::from_utf8_lossy(&child.stderr)
        ))
        .as_error());
    }

    Ok(())
}

pub fn get_processor_arguments<T: AsRef<str>>(
    libraries_path: &Path,
    arguments: &[T],
    data: &HashMap<String, modded::SidedDataEntry>,
) -> crate::Result<Vec<String>> {
    let mut new_arguments = Vec::new();

    for argument in arguments {
        let trimmed_arg = &argument.as_ref()[1..argument.as_ref().len() - 1];
        if argument.as_ref().starts_with('{') {
            if let Some(entry) = data.get(trimmed_arg) {
                new_arguments.push(if entry.client.starts_with('[') {
                    minecraft::get_lib_path(
                        libraries_path,
                        &entry.client[1..entry.client.len() - 1],
                        true,
                    )?
                } else {
                    entry.client.clone()
                })
            }
        } else if argument.as_ref().starts_with('[') {
            new_arguments.push(minecraft::get_lib_path(libraries_path, trimmed_arg, true)?)
        } else {
            new_arguments.push(argument.as_ref().to_string())
        }
    }

    Ok(new_arguments)
}

pub async fn get_processor_main_class(path: String) -> crate::Result<Option<String>> {
    let main_class = tokio::task::spawn_blocking(move || {
        let zip_file = std::fs::File::open(&path).map_err(|e| IOError::with_path(e, &path))?;
        let mut archive = zip::ZipArchive::new(zip_file).map_err(|_| {
            crate::ErrorKind::LauncherError(format!("Cannot read processor at {}", path)).as_error()
        })?;

        let file = archive.by_name("META-INF/MANIFEST.MF").map_err(|_| {
            crate::ErrorKind::LauncherError(format!("Cannot read processor manifest at {}", path))
                .as_error()
        })?;

        let reader = BufReader::new(file);

        for line in reader.lines() {
            let mut line = line.map_err(IOError::from)?;
            line.retain(|c| !c.is_whitespace());

            if line.starts_with("Main-Class:") {
                if let Some(class) = line.split(':').nth(1) {
                    return Ok(Some(class.to_string()));
                }
            }
        }

        Ok::<Option<String>, crate::Error>(None)
    })
    .await??;

    Ok(main_class)
}
