use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use bytes::Bytes;
use serde::Deserialize;

use crate::{
    features::{
        events::{ProgressBarId, ProgressEventType, ProgressService},
        java::ports::JreProvider,
    },
    shared::{Request, RequestClient, RequestClientExt},
};

use super::JAVA_WINDOW_BIN;

#[derive(Deserialize, Clone)]
struct Package {
    pub download_url: String,
    pub name: PathBuf,
}

const AZUL_PACKAGES_BASE_API_URL: &str = "https://api.azul.com/metadata/v1/zulu/packages";

const AZUL_PACKAGES_DEFAULT_QUERY_PARAMS: &str =
    "archive_type=zip&javafx_bundled=false&java_package_type=jre&page_size=1";

pub struct AzulJreProvider<PS: ProgressService, RC: RequestClient> {
    progress_service: Arc<PS>,
    request_client: Arc<RC>,
}

impl<PS: ProgressService, RC: RequestClient> AzulJreProvider<PS, RC> {
    pub fn new(progress_service: Arc<PS>, request_client: Arc<RC>) -> Self {
        Self {
            progress_service,
            request_client,
        }
    }

    fn build_packages_url(version: u32) -> String {
        format!(
          "{AZUL_PACKAGES_BASE_API_URL}?arch={}&java_version={}&os={}&{AZUL_PACKAGES_DEFAULT_QUERY_PARAMS}",
          std::env::consts::ARCH,
          version,
          std::env::consts::OS
      )
    }

    async fn fetch_package(&self, version: u32) -> crate::Result<Package> {
        let packages_url = Self::build_packages_url(version);
        let packages: Vec<Package> = self
            .request_client
            .fetch_json_with_progress(Request::get(packages_url), None)
            .await?;

        packages.first().cloned().ok_or_else(|| {
            crate::ErrorKind::NoValueFor(format!(
                "No Java Version found for Java version {}, OS {}, and Architecture {}",
                version,
                std::env::consts::OS,
                std::env::consts::ARCH,
            ))
            .into()
        })
    }

    async fn download_package(
        &self,
        package: &Package,
        loading_bar_id: &ProgressBarId,
    ) -> crate::Result<Bytes> {
        self.request_client
            .fetch_bytes_with_progress(
                Request::get(&package.download_url),
                Some((loading_bar_id, 80.0)),
            )
            .await
    }

    async fn download_jre(
        &self,
        version: u32,
        loading_bar_id: &ProgressBarId,
    ) -> crate::Result<(Package, Bytes)> {
        self.progress_service
            .emit_progress(loading_bar_id, 0.0, Some("Fetching java version"))
            .await?;
        let package = self.fetch_package(version).await?;

        self.progress_service
            .emit_progress(loading_bar_id, 10.0, Some("Downloading java version"))
            .await?;
        let file = self.download_package(&package, loading_bar_id).await?;

        Ok((package, file))
    }

    fn resolve_java_executable_path(path: &Path, package: &Package, _version: u32) -> PathBuf {
        let base_path = path.join(
            package
                .name
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        );

        #[cfg(target_os = "macos")]
        {
            base_path
                .join(format!("zulu-{}.jre", _version))
                .join("Contents")
                .join("Home")
                .join("bin")
                .join("java")
        }

        #[cfg(not(target_os = "macos"))]
        {
            base_path.join("bin").join(JAVA_WINDOW_BIN)
        }
    }

    async fn unpack_jre(
        &self,
        path: &Path,
        file: Bytes,
        package: &Package,
        version: u32,
        loading_bar_id: &ProgressBarId,
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

        self.progress_service
            .emit_progress(loading_bar_id, 0.0, Some("Extracting java"))
            .await?;
        archive.extract(path).map_err(|_| {
            crate::Error::from(crate::ErrorKind::InputError(
                "Failed to extract java zip".to_string(),
            ))
        })?;

        self.progress_service
            .emit_progress(loading_bar_id, 10.0, Some("Done extracting java"))
            .await?;

        Ok(Self::resolve_java_executable_path(path, package, version))
    }
}

#[async_trait]
impl<PS: ProgressService, RC: RequestClient> JreProvider for AzulJreProvider<PS, RC> {
    async fn install(&self, version: u32, install_dir: &Path) -> crate::Result<PathBuf> {
        let loading_bar_id = self
            .progress_service
            .init_progress(
                ProgressEventType::JavaDownload { version },
                100.0,
                "Downloading java version".to_string(),
            )
            .await?;

        let (package, file) = self.download_jre(version, &loading_bar_id).await?;

        self.unpack_jre(install_dir, file, &package, version, &loading_bar_id)
            .await
    }
}
