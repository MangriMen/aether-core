use std::sync::Arc;

use bytes::Bytes;
use futures::StreamExt;

use crate::{
    features::{
        events::{loading_try_for_each_concurrent, ProgressBarId, ProgressService},
        settings::LocationInfo,
    },
    shared::{read_json_async, write_async, Request, RequestClient, RequestClientExt},
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
        loading_amount: f64,
        loading_bar: Option<&ProgressBarId>,
    ) -> crate::Result<()> {
        log::info!("Downloading assets");

        let assets_stream = futures::stream::iter(index.objects.iter())
            .map(Ok::<(&String, &daedalus::minecraft::Asset), crate::Error>);

        let futures_count = index.objects.len();

        loading_try_for_each_concurrent(
            self.progress_service.clone(),
            assets_stream,
            None,
            loading_bar,
            loading_amount,
            futures_count,
            None,
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
    ) -> crate::Result<()> {
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
                    }).await?;

                    write_async(&asset_path, &asset_resource).await?;
                }

                Ok::<(), crate::Error>(())
            },
            // Download legacy asset
            async {
                let legacy_path = self.location_info.legacy_assets_dir().join(name.replace('/', &String::from(std::path::MAIN_SEPARATOR)));

                if with_legacy && !legacy_path.exists() || force {
                    let asset_resource = fetch_cell.get_or_try_init(|| {
                      self.request_client.fetch_bytes(Request::get(&url))
                    }).await?;

                    write_async(&legacy_path, &asset_resource).await?;
                }

                Ok::<(), crate::Error>(())
            }
        };

        match res {
            Ok(_) => {
                log::debug!("Downloaded asset \"{}\"", name);
                Ok(())
            }
            Err(err) => {
                log::error!("Failed downloading asset \"{}\". err: {}", name, err);
                Err(err)
            }
        }
    }

    pub async fn download_assets_index(
        &self,
        version_info: &daedalus::minecraft::VersionInfo,
        force: bool,
        loading_bar: Option<&ProgressBarId>,
    ) -> crate::Result<daedalus::minecraft::AssetsIndex> {
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
                .await?;

            write_async(&path, &serde_json::to_vec(&assets_index)?).await?;

            Ok(assets_index)
        }?;

        if let Some(loading_bar) = loading_bar {
            self.progress_service
                .emit_progress(loading_bar, 5.0, None)?;
        }

        Ok(res)
    }
}
