use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::{
        events::{ProgressBarId, ProgressConfig, ProgressService, ProgressServiceExt},
        minecraft::{MinecraftDownloader, MinecraftError},
        settings::LocationInfo,
    },
    libs::request_client::{Request, RequestClient, RequestClientExt},
    shared::{read_json_async, write_json_async, IoError},
};

use super::{AssetsService, ClientService, LibrariesService};

pub struct MinecraftDownloadService<RC: RequestClient, PS: ProgressService> {
    client_service: ClientService<RC, PS>,
    assets_service: AssetsService<RC, PS>,
    libraries_service: LibrariesService<RC, PS>,
    location_info: Arc<LocationInfo>,
    request_client: Arc<RC>,
    progress_service: Arc<PS>,
}

impl<RC: RequestClient, PS: ProgressService> MinecraftDownloadService<RC, PS> {
    pub fn new(
        client_service: ClientService<RC, PS>,
        assets_service: AssetsService<RC, PS>,
        libraries_service: LibrariesService<RC, PS>,
        location_info: Arc<LocationInfo>,
        request_client: Arc<RC>,
        progress_service: Arc<PS>,
    ) -> Self {
        Self {
            client_service,
            assets_service,
            libraries_service,
            location_info,
            request_client,
            progress_service,
        }
    }
}

#[async_trait]
impl<RC: RequestClient, PS: ProgressService> MinecraftDownloader
    for MinecraftDownloadService<RC, PS>
{
    async fn download_minecraft(
        &self,
        version_info: &daedalus::minecraft::VersionInfo,
        java_arch: &str,
        force: bool,
        minecraft_updated: bool,
        loading_bar: Option<&ProgressBarId>,
    ) -> Result<(), MinecraftError> {
        log::info!(
            "---------------- Downloading minecraft {} ----------------------------",
            version_info.id
        );

        let assets_index = self
            .assets_service
            .download_assets_index(version_info, force, loading_bar)
            .await?;

        let amount = if version_info
            .processors
            .as_ref()
            .map(|x| !x.is_empty())
            .unwrap_or(false)
        {
            25.0
        } else {
            40.0
        };

        let progress_config = loading_bar.map(|loading_bar| ProgressConfig {
            progress_bar_id: loading_bar,
            total_progress: amount,
        });

        tokio::try_join! {
            self.client_service.download_client(version_info, force, loading_bar),
            self.assets_service.download_assets(&assets_index, version_info.assets == "legacy", force, progress_config.as_ref()),
            self.libraries_service.download_libraries( version_info.libraries.as_slice(), version_info, java_arch, force, minecraft_updated, progress_config.as_ref())
        }?;

        log::info!(
            "---------------- Downloaded minecraft {} successfully ----------------",
            version_info.id
        );

        Ok(())
    }

    async fn download_version_info(
        &self,
        version: &daedalus::minecraft::Version,
        loader: Option<&daedalus::modded::LoaderVersion>,
        force: Option<bool>,
        loading_bar: Option<&ProgressBarId>,
    ) -> Result<daedalus::minecraft::VersionInfo, MinecraftError> {
        let version_id =
            loader.map_or(version.id.clone(), |it| format!("{}-{}", version.id, it.id));

        let path = self
            .location_info
            .version_dir(&version_id)
            .join(format!("{version_id}.json"));

        let res = if path.exists() && !force.unwrap_or(false) {
            read_json_async(path).await
        } else {
            let mut version_info = self
                .request_client
                .fetch_json(Request::get(&version.url))
                .await
                .map_err(|err| {
                    IoError::IOError(std::io::Error::new(
                        std::io::ErrorKind::NetworkUnreachable,
                        err,
                    ))
                })?;

            if let Some(loader) = loader {
                let modded_info = self
                    .request_client
                    .fetch_json(Request::get(&loader.url))
                    .await
                    .map_err(|err| {
                        IoError::IOError(std::io::Error::new(
                            std::io::ErrorKind::NetworkUnreachable,
                            err,
                        ))
                    })?;

                version_info = daedalus::modded::merge_partial_version(modded_info, version_info);
            }

            version_info.id.clone_from(&version_id);

            write_json_async(&path, &version_info).await?;

            Ok(version_info)
        }?;

        if let Some(loading_bar) = loading_bar {
            self.progress_service
                .emit_progress_safe(loading_bar, 5.0, None)
                .await;
        }

        Ok(res)
    }
}
