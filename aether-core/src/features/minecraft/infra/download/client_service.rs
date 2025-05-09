use std::sync::Arc;

use crate::{
    features::{
        events::{ProgressBarId, ProgressService},
        settings::LocationInfo,
    },
    shared::{write_async, Request, RequestClient},
};

pub struct ClientService<RC: RequestClient, PS: ProgressService> {
    progress_service: Arc<PS>,
    request_client: Arc<RC>,
    location_info: Arc<LocationInfo>,
}

impl<RC: RequestClient, PS: ProgressService> ClientService<RC, PS> {
    pub fn new(
        progress_service: Arc<PS>,
        request_client: Arc<RC>,
        location_info: Arc<LocationInfo>,
    ) -> Self {
        Self {
            progress_service,
            request_client,
            location_info,
        }
    }

    pub async fn download_client(
        &self,
        version_info: &daedalus::minecraft::VersionInfo,
        force: bool,
        loading_bar: Option<&ProgressBarId>,
    ) -> crate::Result<()> {
        log::info!("Downloading client {}", version_info.id);
        let version_id = &version_info.id;

        let client_download_url = version_info
            .downloads
            .get(&daedalus::minecraft::DownloadType::Client)
            .ok_or(
                crate::ErrorKind::LauncherError(format!(
                    "No client downloads exist for version {version_id}"
                ))
                .as_error(),
            )?;

        let path = self
            .location_info
            .version_dir(version_id)
            .join(format!("{version_id}.jar"));

        if !path.exists() || force {
            let bytes = self
                .request_client
                .fetch_bytes(Request::get(&client_download_url.url))
                .await?;
            write_async(&path, &bytes).await?;
        }

        if let Some(loading_bar) = loading_bar {
            self.progress_service
                .emit_progress(loading_bar, 9.0, None)
                .await?;
        }

        log::info!("Downloaded client {} successfully", version_info.id);

        Ok(())
    }
}
