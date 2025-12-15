use std::sync::Arc;

use bytes::Bytes;
use futures::StreamExt;

use crate::{
    features::{
        events::{
            try_for_each_concurrent_with_progress, ProgressBarId, ProgressConfig,
            ProgressConfigWithMessage, ProgressService, ProgressServiceExt,
        },
        minecraft::MinecraftError,
        settings::LocationInfo,
    },
    libs::request_client::{Request, RequestClient, RequestClientExt},
    shared::{read_json_async, write_async, write_json_async, IoError},
};

const MINECRAFT_RESOURCES_BASE_URL: &str = "https://resources.download.minecraft.net/";

pub struct AssetsService<RC: RequestClient, PS: ProgressService> {
    progress_service: Arc<PS>,
    request_client: Arc<RC>,
    location_info: Arc<LocationInfo>,
}

impl<RC: RequestClient, PS: ProgressService> AssetsService<RC, PS> {
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

    pub async fn download_assets(
        &self,
        index: &daedalus::minecraft::AssetsIndex,
        with_legacy: bool,
        force: bool,
        progress_config: Option<&ProgressConfig<'_>>,
    ) -> Result<(), MinecraftError> {
        log::info!("Downloading assets");

        let assets_stream = futures::stream::iter(index.objects.iter())
            .map(Ok::<(&String, &daedalus::minecraft::Asset), MinecraftError>);

        let futures_count = index.objects.len();

        let progress_config = progress_config
            .as_ref()
            .map(|config| ProgressConfigWithMessage {
                progress_bar_id: config.progress_bar_id,
                total_progress: config.total_progress,
                progress_message: None,
            });

        try_for_each_concurrent_with_progress(
            self.progress_service.clone(),
            assets_stream,
            None,
            futures_count,
            progress_config.as_ref(),
            |(name, asset)| async move {
                log::debug!("Downloading asset \"{}\"", name);
                let res = self.download_asset(name, asset, with_legacy, force).await;
                match res {
                    Ok(res) => {
                        log::debug!("Downloaded asset \"{}\"", name);
                        Ok(res)
                    }
                    Err(err) => {
                        log::error!("Failed downloading asset \"{}\". err: {}", name, err);
                        Err(err)
                    }
                }
            },
        )
        .await?;

        log::info!("Downloaded assets successfully");

        Ok(())
    }

    pub async fn download_asset(
        &self,
        name: &String,
        asset: &daedalus::minecraft::Asset,
        with_legacy: bool,
        force: bool,
    ) -> Result<(), MinecraftError> {
        let hash = &asset.hash;
        let url = format!(
            "{MINECRAFT_RESOURCES_BASE_URL}{sub_hash}/{hash}",
            sub_hash = &hash[..2]
        );
        log::trace!("Downloading asset \"{}\"\n\tfrom {}", name, url);

        let asset_path = self.location_info.object_dir(hash);

        let fetch_cell = tokio::sync::OnceCell::<Bytes>::new();

        let res = tokio::try_join! {
            // Download asset
            async  {
                if !asset_path.exists() || force {
                    let asset_resource = fetch_cell.get_or_try_init(|| {
                      self.request_client.fetch_bytes(Request::get(&url))
                    })
                    .await
                    .map_err(|err| IoError::IoError(std::io::Error::new(
                        std::io::ErrorKind::NetworkUnreachable,
                        err,
                    )))?;

                    write_async(&asset_path, &asset_resource).await?;
                }

                Ok::<(), MinecraftError>(())
            },
            // Download legacy asset
            async {
                let legacy_path = self.location_info.legacy_assets_dir().join(name.replace('/', &String::from(std::path::MAIN_SEPARATOR)));

                if with_legacy && !legacy_path.exists() || force {
                    let asset_resource = fetch_cell.get_or_try_init(|| {
                      self.request_client.fetch_bytes(Request::get(&url))
                    })
                    .await
                    .map_err(|err| IoError::IoError(std::io::Error::new(
                        std::io::ErrorKind::NetworkUnreachable,
                        err,
                    )))?;

                    write_async(&legacy_path, &asset_resource).await?;
                }

                Ok::<(), MinecraftError>(())
            }
        };

        match res {
            Ok(_) => {
                log::debug!("Downloaded asset \"{}\"", name);
                Ok(())
            }
            Err(err) => {
                log::error!("Failed downloading asset \"{}\". err: {}", name, err);
                Ok(Err(IoError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NetworkUnreachable,
                    err,
                )))?)
            }
        }
    }

    pub async fn download_assets_index(
        &self,
        version_info: &daedalus::minecraft::VersionInfo,
        force: bool,
        loading_bar: Option<&ProgressBarId>,
    ) -> Result<daedalus::minecraft::AssetsIndex, MinecraftError> {
        let path = self
            .location_info
            .assets_index_dir()
            .join(format!("{}.json", &version_info.asset_index.id));

        let res = if path.exists() && !force {
            read_json_async(path).await
        } else {
            let assets_index = self
                .request_client
                .fetch_json(Request::get(&version_info.asset_index.url))
                .await
                .map_err(|err| {
                    IoError::IoError(std::io::Error::new(
                        std::io::ErrorKind::NetworkUnreachable,
                        err,
                    ))
                })?;

            write_json_async(&path, &assets_index).await?;

            Ok(assets_index)
        }?;

        if let Some(loading_bar) = loading_bar {
            self.progress_service
                .emit_progress_safe(loading_bar, 5.0, None)
                .await;
        }

        Ok(res)
    }
}
