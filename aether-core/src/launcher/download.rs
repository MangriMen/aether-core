use crate::{
    state::LauncherState,
    utils::{self, fetch::fetch_advanced, io::write_async},
};
use anyhow::Context;
use bytes::Bytes;
use daedalus::minecraft::{self, AssetsIndex};
use futures::{stream, StreamExt, TryStreamExt};
use reqwest::Method;

const MINECRAFT_RESOURCE_BASE_URL: &str = "https://resources.download.minecraft.net";

#[tracing::instrument]
pub async fn download_minecraft(
    state: &LauncherState,
    version_info: &minecraft::VersionInfo,
    force: bool,
) -> anyhow::Result<()> {
    let assets_index = &download_assets_index(state, version_info, force).await?;

    tokio::try_join! {
        download_client(state, version_info, force),
        download_assets(state, assets_index, version_info.assets == "legacy", force)
        //  download libraries -> @download_libraries
    }?;

    Ok(())
}

#[tracing::instrument]
pub async fn download_assets_index(
    state: &LauncherState,
    version_info: &minecraft::VersionInfo,
    force: bool,
) -> anyhow::Result<AssetsIndex> {
    let path = state
        .locations
        .assets_index_dir()
        .join(format!("{}.json", &version_info.asset_index.id));

    let res = if path.exists() && !force {
        utils::io::read_json_async(path).await
    } else {
        let assets_index =
            utils::fetch::fetch_json(Method::GET, &version_info.asset_index.url, None, None)
                .await?;

        utils::io::write_async(&path, &serde_json::to_vec(&assets_index)?).await?;

        Ok(assets_index)
    }?;

    Ok(res)
}

#[tracing::instrument]
pub async fn download_asset(
    state: &LauncherState,
    name: &String,
    asset: &minecraft::Asset,
    with_legacy: bool,
    force: bool,
) -> anyhow::Result<()> {
    let hash = &asset.hash;
    let url = format!(
        "{MINECRAFT_RESOURCE_BASE_URL}/{sub_hash}/{hash}",
        sub_hash = &hash[..2]
    );

    let asset_path = state.locations.object_dir(hash);

    let fetch_cell = tokio::sync::OnceCell::<Bytes>::new();

    tokio::try_join! {
        // Download asset
        async  {
            if !asset_path.exists() || force {
                let asset_resource = fetch_cell.get_or_try_init(|| {
                    utils::fetch::fetch_advanced(
                        Method::GET,
                        &url,
                        None,
                        None
                    )
                }).await?;

                utils::io::write_async(&asset_path, &asset_resource).await?;
            }

            Ok::<(), anyhow::Error>(())
        },
        // Download legacy asset
        async {
            let legacy_path = state.locations.legacy_assets_dir().join(name.replace('/', &String::from(std::path::MAIN_SEPARATOR)));

            if with_legacy && !legacy_path.exists() || force {
                let asset_resource = fetch_cell.get_or_try_init(|| {
                    utils::fetch::fetch_advanced(
                        Method::GET,
                        &url,
                        None,
                        None
                    )
                }).await?;

                utils::io::write_async(&legacy_path, &asset_resource).await?;
            }

            Ok::<(), anyhow::Error>(())
        }
    }?;

    Ok(())
}

#[tracing::instrument]
pub async fn download_assets(
    state: &LauncherState,
    index: &minecraft::AssetsIndex,
    with_legacy: bool,
    force: bool,
) -> anyhow::Result<()> {
    let assets =
        stream::iter(index.objects.iter()).map(Ok::<(&String, &minecraft::Asset), anyhow::Error>);

    assets
        .try_for_each_concurrent(None, |(name, asset)| async move {
            download_asset(state, name, asset, with_legacy, force).await
        })
        .await?;

    Ok(())
}

pub async fn download_client(
    state: &LauncherState,
    version_info: &minecraft::VersionInfo,
    force: bool,
) -> anyhow::Result<()> {
    let version_id = &version_info.id;

    let client_download = version_info
        .downloads
        .get(&minecraft::DownloadType::Client)
        .context(format!(
            "No client downloads exists for version {version_id}"
        ))?;

    let path = state
        .locations
        .version_dir(version_id)
        .join(format!("{version_id}.jar"));

    if !path.exists() || force {
        let bytes = fetch_advanced(Method::GET, &client_download.url, None, None).await?;
        write_async(&path, &bytes).await?;
    }

    Ok(())
}
