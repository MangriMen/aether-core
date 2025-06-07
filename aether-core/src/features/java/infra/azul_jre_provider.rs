use std::{
    io::Cursor,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use bytes::Bytes;
use serde::Deserialize;

use crate::{
    features::{
        events::{ProgressBarId, ProgressEventType, ProgressService, ProgressServiceExt},
        java::{ports::JreProvider, JavaError},
    },
    libs::request_client::{Request, RequestClient, RequestClientExt},
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
            "{AZUL_PACKAGES_BASE_API_URL}?arch={arch}&java_version={version}&os={os}&{params}",
            arch = std::env::consts::ARCH,
            version = version,
            os = std::env::consts::OS,
            params = AZUL_PACKAGES_DEFAULT_QUERY_PARAMS
        )
    }

    async fn fetch_package(&self, version: u32) -> Result<Package, JavaError> {
        let packages_url = Self::build_packages_url(version);
        let packages: Vec<Package> = self
            .request_client
            .fetch_json_with_progress(Request::get(packages_url), None)
            .await?;

        packages
            .first()
            .cloned()
            .ok_or_else(|| JavaError::JavaDownloadNotFound {
                version,
                os: std::env::consts::OS.to_owned(),
                arch: std::env::consts::ARCH.to_owned(),
            })
    }

    async fn download_package(
        &self,
        package: &Package,
        progress_bar_id: Option<&ProgressBarId>,
    ) -> Result<Bytes, JavaError> {
        self.request_client
            .fetch_bytes_with_progress(
                Request::get(&package.download_url),
                progress_bar_id.map(|progress_bar_id| (progress_bar_id, 80.0)),
            )
            .await
            .map_err(Into::into)
    }

    async fn download_jre(
        &self,
        version: u32,
        progress_bar_id: Option<&ProgressBarId>,
    ) -> Result<(Package, Bytes), JavaError> {
        if let Some(progress_bar_id) = progress_bar_id {
            self.progress_service
                .emit_progress_safe(progress_bar_id, 0.0, Some("Fetching java version"))
                .await;
        }

        let package = self.fetch_package(version).await?;

        if let Some(progress_bar_id) = progress_bar_id {
            self.progress_service
                .emit_progress_safe(progress_bar_id, 10.0, Some("Downloading java version"))
                .await;
        }

        let file = self.download_package(&package, progress_bar_id).await?;

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

    async fn remove_old_jre_installation(path: &Path) -> Result<(), JavaError> {
        if path.exists() {
            tokio::fs::remove_dir_all(&path).await.map_err(|_| {
                JavaError::RemoveOldInstallationError {
                    path: path.to_path_buf(),
                }
            })?;
        }

        Ok(())
    }

    async fn unpack_jre(
        &self,
        path: &Path,
        file: Bytes,
        package: &Package,
        version: u32,
        progress_bar_id: Option<&ProgressBarId>,
    ) -> Result<PathBuf, JavaError> {
        let mut archive = zip::ZipArchive::new(Cursor::new(file))?;

        if let Some(file) = archive.file_names().next() {
            if let Some(dir) = file.split('/').next() {
                Self::remove_old_jre_installation(&path.join(dir)).await?;
            }
        }

        if let Some(progress_bar_id) = progress_bar_id {
            self.progress_service
                .emit_progress_safe(progress_bar_id, 0.0, Some("Extracting java"))
                .await;
        }

        archive.extract(path)?;

        if let Some(progress_bar_id) = progress_bar_id {
            self.progress_service
                .emit_progress_safe(progress_bar_id, 10.0, Some("Done extracting java"))
                .await;
        }

        Ok(Self::resolve_java_executable_path(path, package, version))
    }
}

#[async_trait]
impl<PS: ProgressService, RC: RequestClient> JreProvider for AzulJreProvider<PS, RC> {
    async fn install(&self, version: u32, install_dir: &Path) -> Result<PathBuf, JavaError> {
        let progress_bar_id = self
            .progress_service
            .init_progress_safe(
                ProgressEventType::JavaDownload { version },
                100.0,
                "Downloading java version".to_string(),
            )
            .await;

        let (package, file) = self.download_jre(version, progress_bar_id.as_ref()).await?;

        self.unpack_jre(
            install_dir,
            file,
            &package,
            version,
            progress_bar_id.as_ref(),
        )
        .await
    }
}
