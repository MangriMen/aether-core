use std::path::PathBuf;

use reqwest::Method;

use crate::{
    event::{emit_loading, init_loading, LoadingBarType},
    state::LauncherState,
    utils::fetch::{fetch_advanced, fetch_json},
};

pub enum JavaProviders {
    Azul,
}

pub fn get_java_download_url(
    provider: JavaProviders,
    arch: &str,
    java_version: u32,
    os: &str,
) -> String {
    match provider {
        JavaProviders::Azul => format!(
            "https://api.azul.com/metadata/v1/zulu/packages?arch={}&java_version={}&os={}&archive_type=zip&javafx_bundled=false&java_package_type=jre&page_size=1",
            arch, java_version, os
        )
    }
}

pub async fn auto_install_java(java_version: u32) -> crate::Result<PathBuf> {
    let state = LauncherState::get().await?;

    let loading_bar = init_loading(
        LoadingBarType::JavaDownload {
            version: java_version,
        },
        100.0,
        "Downloading java version",
    )
    .await?;

    #[derive(serde::Deserialize)]
    struct Package {
        pub download_url: String,
        pub name: PathBuf,
    }

    emit_loading(&loading_bar, 0.0, Some("Fetching java version")).await?;

    let packages = fetch_json::<Vec<Package>>(
        Method::GET,
        &get_java_download_url(
            JavaProviders::Azul,
            std::env::consts::ARCH,
            java_version,
            std::env::consts::OS,
        ),
        None,
        None,
        None,
        &state.fetch_semaphore,
    )
    .await?;

    emit_loading(&loading_bar, 10.0, Some("Downloading java version")).await?;

    if let Some(download) = packages.first() {
        let file = fetch_advanced(
            Method::GET,
            &download.download_url,
            None,
            None,
            None,
            &state.fetch_semaphore,
            Some((&loading_bar, 80.0)),
        )
        .await?;

        let path = state.locations.java_dir();

        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(file)).map_err(|_| {
            crate::Error::from(crate::ErrorKind::InputError(
                "Failed to read java zip".to_string(),
            ))
        })?;

        // removes the old installation of java
        if let Some(file) = archive.file_names().next() {
            if let Some(dir) = file.split('/').next() {
                let path = path.join(dir);

                if path.exists() {
                    tokio::fs::remove_dir_all(path).await?;
                }
            }
        }

        emit_loading(&loading_bar, 0.0, Some("Extracting java")).await?;

        archive.extract(&path).map_err(|_| {
            crate::Error::from(crate::ErrorKind::InputError(
                "Failed to extract java zip".to_string(),
            ))
        })?;

        emit_loading(&loading_bar, 10.0, Some("Done extracting java")).await?;

        let mut base_path = path.join(
            download
                .name
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        );

        #[cfg(target_os = "macos")]
        {
            base_path = base_path
                .join(format!("zulu-{}.jre", java_version))
                .join("Contents")
                .join("Home")
                .join("bin")
                .join("java")
        }

        #[cfg(not(target_os = "macos"))]
        {
            base_path = base_path.join("bin").join(crate::utils::jre::JAVA_BIN)
        }

        Ok(base_path)
    } else {
        Err(crate::ErrorKind::LauncherError(format!(
            "No Java Version found for Java version {}, OS {}, and Architecture {}",
            java_version,
            std::env::consts::OS,
            std::env::consts::ARCH,
        ))
        .into())
    }
}
