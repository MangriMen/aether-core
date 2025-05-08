use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    path::Path,
    sync::Arc,
};

use async_trait::async_trait;
use daedalus::minecraft::VersionInfo;
use tokio::process::Command;

use crate::{
    features::{
        events::{EventEmitter, ProgressBarId, ProgressBarStorage, ProgressService},
        instance::Instance,
        java::Java,
        minecraft::{self, ModLoaderProcessor},
        settings::LocationInfo,
    },
    processor_rules,
    shared::IOError,
    with_mut_ref,
};

pub struct ForgeProcessor<E: EventEmitter, PBS: ProgressBarStorage> {
    progress_service: Arc<ProgressService<E, PBS>>,
    location_info: Arc<LocationInfo>,
}

impl<E: EventEmitter, PBS: ProgressBarStorage> ForgeProcessor<E, PBS> {
    pub fn new(
        progress_service: Arc<ProgressService<E, PBS>>,
        location_info: Arc<LocationInfo>,
    ) -> Self {
        Self {
            progress_service,
            location_info,
        }
    }

    async fn run_single_processor(
        processor: &daedalus::modded::Processor,
        data: &HashMap<String, daedalus::modded::SidedDataEntry>,
        libraries_dir: &Path,
        java_version: &Java,
    ) -> crate::Result<()> {
        log::debug!("Running forge processor {}", processor.jar);

        let class_path: Vec<String> = with_mut_ref!(cp = processor.classpath.clone() => {
            cp.push(processor.jar.clone())
        });

        let class_path_arg =
            minecraft::get_class_paths_jar(libraries_dir, &class_path, &java_version.architecture)?;

        let processor_jar_path = minecraft::get_lib_path(libraries_dir, &processor.jar, false)?;
        let processor_main_class = get_processor_main_class(processor_jar_path)
            .await?
            .ok_or_else(|| {
                crate::ErrorKind::LauncherError(format!(
                    "Could not find processor main class for {}",
                    processor.jar
                ))
            })?;

        let processor_args = get_processor_arguments(libraries_dir, &processor.args, data)?;

        let output = Command::new(&java_version.path)
            .arg("-cp")
            .arg(class_path_arg)
            .arg(processor_main_class)
            .args(processor_args)
            .output()
            .await
            .map_err(|e| IOError::with_path(e, &java_version.path))
            .map_err(|err| {
                crate::ErrorKind::LauncherError(format!("Error running processor: {err}"))
            })?;

        if !output.status.success() {
            return Err(crate::ErrorKind::LauncherError(format!(
                "Processor error: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
            .as_error());
        }

        Ok(())
    }
}

#[async_trait]
impl<E: EventEmitter, PBS: ProgressBarStorage> ModLoaderProcessor for ForgeProcessor<E, PBS> {
    async fn run(
        &self,
        instance: &Instance,
        version_jar: String,
        instance_path: &Path,
        version_info: &mut VersionInfo,
        java_version: &Java,
        loading_bar: Option<&ProgressBarId>,
    ) -> crate::Result<()> {
        let Some(processors) = &version_info.processors else {
            return Ok(());
        };

        let client_path = self
            .location_info
            .version_dir(&version_jar)
            .join(format!("{version_jar}.jar"));

        let libraries_dir = self.location_info.libraries_dir();

        let Some(ref mut data) = version_info.data else {
            return Ok(());
        };

        processor_rules! {
            data;
            "SIDE":
                client => "client",
                server => "";
            "MINECRAFT_JAR":
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
            self.progress_service.emit_progress(
                loading_bar,
                0.0,
                Some("Running forge processors"),
            )?;
        }

        let total_processors = processors.len();
        for (index, processor) in processors.iter().enumerate() {
            if let Some(sides) = &processor.sides {
                if !sides.contains(&String::from("client")) {
                    continue;
                }
            }

            Self::run_single_processor(processor, data, &libraries_dir, java_version).await?;

            if let Some(loading_bar) = loading_bar {
                let progress = 30.0 / total_processors as f64;
                let message = format!("Running forge processor {}/{}", index + 1, total_processors);
                self.progress_service
                    .emit_progress(loading_bar, progress, Some(&message))?;
            }
        }

        Ok(())
    }
}

fn process_argument(
    libraries_path: &Path,
    argument: &str,
    data: &HashMap<String, daedalus::modded::SidedDataEntry>,
) -> crate::Result<String> {
    if argument.starts_with('{') {
        let key = &argument[1..argument.len() - 1];
        data.get(key)
            .map(|entry| {
                if entry.client.starts_with('[') {
                    minecraft::get_lib_path(
                        libraries_path,
                        &entry.client[1..entry.client.len() - 1],
                        true,
                    )
                } else {
                    Ok(entry.client.clone())
                }
            })
            .transpose()?
            .ok_or_else(|| {
                crate::ErrorKind::LauncherError(format!("Missing data entry for key: {}", key))
                    .as_error()
            })
    } else if argument.starts_with('[') {
        let lib_path = &argument[1..argument.len() - 1];
        minecraft::get_lib_path(libraries_path, lib_path, true)
    } else {
        Ok(argument.to_string())
    }
}

pub fn get_processor_arguments<T: AsRef<str>>(
    libraries_path: &Path,
    arguments: &[T],
    data: &HashMap<String, daedalus::modded::SidedDataEntry>,
) -> crate::Result<Vec<String>> {
    arguments
        .iter()
        .map(|arg| process_argument(libraries_path, arg.as_ref(), data))
        .collect()
}

pub async fn get_processor_main_class(path: String) -> crate::Result<Option<String>> {
    tokio::task::spawn_blocking(move || {
        let file = std::fs::File::open(&path).map_err(|e| IOError::with_path(e, &path))?;
        let mut archive = zip::ZipArchive::new(file).map_err(|_| {
            crate::ErrorKind::LauncherError(format!("Cannot read processor at {}", path)).as_error()
        })?;

        let manifest = archive.by_name("META-INF/MANIFEST.MF").map_err(|_| {
            crate::ErrorKind::LauncherError(format!("Cannot read processor manifest at {}", path))
                .as_error()
        })?;

        let reader = BufReader::new(manifest);

        for line in reader.lines() {
            let line = line.map_err(IOError::from)?;
            let trimmed_line = line.trim();

            if let Some(class) = trimmed_line.strip_prefix("Main-Class:") {
                return Ok(Some(class.trim().to_string()));
            }
        }

        Ok(None)
    })
    .await?
}
