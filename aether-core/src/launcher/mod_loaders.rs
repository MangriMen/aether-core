use std::path::PathBuf;

use daedalus::minecraft::VersionInfo;
use tokio::process::Command;

use crate::{
    event::{emit_loading, LoadingBarId},
    launcher::args,
    state::{Instance, Java, LauncherState},
    utils::io::IOError,
    wrap_ref_builder,
};

#[tracing::instrument]
pub async fn process_forge_processors(
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
            super::processor_rules! {
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

                let cp = wrap_ref_builder!(cp = processor.classpath.clone() => {
                    cp.push(processor.jar.clone())
                });

                let child = Command::new(&java_version.path)
                    .arg("-cp")
                    .arg(args::get_class_paths_jar(
                        &libraries_dir,
                        &cp,
                        &java_version.architecture,
                    )?)
                    .arg(
                        args::get_processor_main_class(args::get_lib_path(
                            &libraries_dir,
                            &processor.jar,
                            false,
                        )?)
                        .await?
                        .ok_or_else(|| {
                            crate::ErrorKind::LauncherError(format!(
                                "Could not find processor main class for {}",
                                processor.jar
                            ))
                        })?,
                    )
                    .args(args::get_processor_arguments(
                        &libraries_dir,
                        &processor.args,
                        data,
                    )?)
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
