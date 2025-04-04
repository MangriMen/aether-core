use std::path::{Path, PathBuf};

use bytes::Bytes;
use reqwest::Method;

use crate::{
    event::{
        emit::{emit_loading, init_loading},
        LoadingBarId, LoadingBarType,
    },
    features::java::constants::JAVA_WINDOW_BIN,
    state::LauncherState,
    utils::fetch::{fetch_advanced, fetch_json},
};

#[derive(serde::Deserialize, Clone)]
struct Package {
    pub download_url: String,
    pub name: PathBuf,
}

async fn download_jre(
    state: &LauncherState,
    version: u32,
    loading_bar_id: &LoadingBarId,
) -> crate::Result<(Package, Bytes)> {
    emit_loading(loading_bar_id, 0.0, Some("Fetching java version")).await?;

    let packages_url = format!(
      "https://api.azul.com/metadata/v1/zulu/packages?arch={}&java_version={}&os={}&archive_type=zip&javafx_bundled=false&java_package_type=jre&page_size=1",
      std::env::consts::ARCH, version, std::env::consts::OS
    );

    let packages = fetch_json::<Vec<Package>>(
        Method::GET,
        &packages_url,
        None,
        None,
        None,
        &state.fetch_semaphore,
    )
    .await?;

    emit_loading(loading_bar_id, 10.0, Some("Downloading java version")).await?;

    if let Some(package) = packages.first() {
        let file = fetch_advanced(
            Method::GET,
            &package.download_url,
            None,
            None,
            None,
            &state.fetch_semaphore,
            Some((loading_bar_id, 80.0)),
        )
        .await?;

        Ok((package.clone(), file))
    } else {
        Err(crate::ErrorKind::NoValueFor(format!(
            "No Java Version found for Java version {}, OS {}, and Architecture {}",
            version,
            std::env::consts::OS,
            std::env::consts::ARCH,
        ))
        .as_error())
    }
}

async fn unpack_jre(
    package: &Package,
    file: Bytes,
    path: &Path,
    loading_bar_id: &LoadingBarId,
) -> crate::Result<PathBuf> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(file)).map_err(|_| {
        crate::Error::from(crate::ErrorKind::InputError(
            "Failed to read java zip".to_string(),
        ))
    })?;

    // Remove the old installation of jre
    if let Some(file) = archive.file_names().next() {
        if let Some(dir) = file.split('/').next() {
            let path = path.join(dir);

            if path.exists() {
                tokio::fs::remove_dir_all(path).await?;
            }
        }
    }

    emit_loading(loading_bar_id, 0.0, Some("Extracting java")).await?;

    archive.extract(path).map_err(|_| {
        crate::Error::from(crate::ErrorKind::InputError(
            "Failed to extract java zip".to_string(),
        ))
    })?;

    emit_loading(loading_bar_id, 10.0, Some("Done extracting java")).await?;

    let mut base_path = path.join(
        package
            .name
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
    );

    #[cfg(target_os = "macos")]
    {
        base_path = base_path
            .join(format!("zulu-{}.jre", version))
            .join("Contents")
            .join("Home")
            .join("bin")
            .join("java")
    }

    #[cfg(not(target_os = "macos"))]
    {
        base_path = base_path.join("bin").join(JAVA_WINDOW_BIN)
    }

    Ok(base_path)
}

pub async fn install_jre(state: &LauncherState, version: u32) -> crate::Result<PathBuf> {
    let loading_bar_id = init_loading(
        LoadingBarType::JavaDownload { version },
        100.0,
        "Downloading java version",
    )
    .await?;

    let (package, file) = download_jre(state, version, &loading_bar_id).await?;

    let path = state.locations.java_dir();
    let base_path = unpack_jre(&package, file, &path, &loading_bar_id).await?;

    Ok(base_path)
}
