use std::sync::Arc;

use bytes::Bytes;

use crate::{
    features::{
        events::{ProgressBarId, ProgressService, ProgressServiceExt},
        minecraft::MinecraftDomainError,
    },
    libs::request_client::{Request, RequestClient},
    shared::{FileStore, IoError},
};

use super::version_jar_key;

pub struct ClientService<RC: RequestClient, PS: ProgressService, FS: FileStore> {
    progress_service: Arc<PS>,
    request_client: Arc<RC>,
    file_store: Arc<FS>,
}

impl<RC: RequestClient, PS: ProgressService, FS: FileStore> ClientService<RC, PS, FS> {
    pub fn new(progress_service: Arc<PS>, request_client: Arc<RC>, file_store: Arc<FS>) -> Self {
        Self {
            progress_service,
            request_client,
            file_store,
        }
    }

    async fn download_client(
        &self,
        version_id: &String,
        version_info: &daedalus::minecraft::VersionInfo,
    ) -> Result<Bytes, MinecraftDomainError> {
        let client_download_url = version_info
            .downloads
            .get(&daedalus::minecraft::DownloadType::Client)
            .ok_or(MinecraftDomainError::VersionNotFound {
                version: version_id.to_owned(),
            })?;

        Ok(self
            .request_client
            .fetch_bytes(Request::get(&client_download_url.url))
            .await
            .map_err(|err| {
                IoError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NetworkUnreachable,
                    err,
                ))
            })?)
    }

    pub async fn ensure_client_download(
        &self,
        version_info: &daedalus::minecraft::VersionInfo,
        force: bool,
        loading_bar: Option<&ProgressBarId>,
    ) -> Result<(), MinecraftDomainError> {
        log::info!("Downloading client {}", version_info.id);

        let version_id = &version_info.id;

        let key = version_jar_key(version_id.to_string());

        if !self.file_store.exists(&key).await || force {
            let bytes = self.download_client(version_id, version_info).await?;
            self.file_store.write(&key, bytes).await;
        }

        if let Some(loading_bar) = loading_bar {
            self.progress_service
                .emit_progress_safe(loading_bar, 9.0, None)
                .await;
        }

        log::info!("Downloaded client {} successfully", version_info.id);

        Ok(())
    }
}
