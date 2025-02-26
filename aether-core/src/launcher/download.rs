use crate::{
    event::{emit::emit_loading, loading_try_for_each_concurrent, LoadingBarId},
    state::LauncherState,
    utils::{self, io::write_async},
};
use bytes::Bytes;
use daedalus::{
    minecraft::{self, AssetsIndex},
    modded::{self},
};
use futures::{stream, StreamExt};
use reqwest::Method;

use crate::config::META_URL;

use super::library::parse_rules;

const MINECRAFT_RESOURCES_BASE_URL: &str = "https://resources.download.minecraft.net/";
const MINECRAFT_LIBRARIES_BASE_URL: &str = "https://libraries.minecraft.net/";

#[tracing::instrument(skip(state, version_info))]
pub async fn download_minecraft(
    state: &LauncherState,
    version_info: &minecraft::VersionInfo,
    java_arch: &str,
    force: bool,
    minecraft_updated: bool,
    loading_bar: Option<&LoadingBarId>,
) -> crate::Result<()> {
    log::info!(
        "---------------- Downloading minecraft {} ----------------------------",
        version_info.id
    );

    let assets_index = &download_assets_index(state, version_info, force, loading_bar).await?;

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

    tokio::try_join! {
        download_client(state, version_info, force, loading_bar),
        download_assets(state, assets_index, version_info.assets == "legacy", force, amount, loading_bar),
        download_libraries(state, version_info.libraries.as_slice(), version_info, java_arch, force, minecraft_updated, amount, loading_bar)
    }?;

    log::info!(
        "---------------- Downloaded minecraft {} successfully ----------------",
        version_info.id
    );

    Ok(())
}

#[tracing::instrument]
pub async fn download_assets_index(
    state: &LauncherState,
    version_info: &minecraft::VersionInfo,
    force: bool,
    loading_bar: Option<&LoadingBarId>,
) -> crate::Result<AssetsIndex> {
    let path = state
        .locations
        .assets_index_dir()
        .join(format!("{}.json", &version_info.asset_index.id));

    let res = if path.exists() && !force {
        utils::io::read_json_async(path).await
    } else {
        let assets_index = utils::fetch::fetch_json(
            Method::GET,
            &version_info.asset_index.url,
            None,
            None,
            None,
            &state.fetch_semaphore,
        )
        .await?;

        utils::io::write_async(&path, &serde_json::to_vec(&assets_index)?).await?;

        Ok(assets_index)
    }?;

    if let Some(loading_bar) = loading_bar {
        emit_loading(loading_bar, 5.0, None).await?;
    }

    Ok(res)
}

#[tracing::instrument(skip_all, fields(version = version.id.as_str() ,loader = ?loader))]
pub async fn download_version_info(
    state: &LauncherState,
    version: &minecraft::Version,
    loader: Option<&modded::LoaderVersion>,
    force: Option<bool>,
    loading_bar: Option<&LoadingBarId>,
) -> crate::Result<minecraft::VersionInfo> {
    let version_id = loader.map_or(version.id.clone(), |it| format!("{}-{}", version.id, it.id));

    let path = state
        .locations
        .version_dir(&version_id)
        .join(format!("{version_id}.json"));

    let res = if path.exists() && !force.unwrap_or(false) {
        utils::io::read_json_async(path).await
    } else {
        let mut version_info = utils::fetch::fetch_json(
            Method::GET,
            &version.url,
            None,
            None,
            None,
            &state.fetch_semaphore,
        )
        .await?;

        if let Some(loader) = loader {
            let modded_info = utils::fetch::fetch_json(
                Method::GET,
                &loader.url,
                None,
                None,
                None,
                &state.fetch_semaphore,
            )
            .await?;
            version_info = modded::merge_partial_version(modded_info, version_info);
        }

        version_info.id.clone_from(&version_id);

        utils::io::write_async(&path, &serde_json::to_vec(&version_info)?).await?;

        Ok(version_info)
    }?;

    if let Some(loading_bar) = loading_bar {
        emit_loading(loading_bar, 5.0, None).await?;
    }

    Ok(res)
}

#[tracing::instrument(skip_all)]
pub async fn download_client(
    state: &LauncherState,
    version_info: &minecraft::VersionInfo,
    force: bool,
    loading_bar: Option<&LoadingBarId>,
) -> crate::Result<()> {
    log::info!("Downloading client {}", version_info.id);

    let version_id = &version_info.id;

    let client_download = version_info
        .downloads
        .get(&minecraft::DownloadType::Client)
        .ok_or(
            crate::ErrorKind::LauncherError(format!(
                "No client downloads exist for version {version_id}"
            ))
            .as_error(),
        )?;

    let path = state
        .locations
        .version_dir(version_id)
        .join(format!("{version_id}.jar"));

    if !path.exists() || force {
        let bytes = utils::fetch::fetch_advanced(
            Method::GET,
            &client_download.url,
            None,
            None,
            None,
            &state.fetch_semaphore,
            None,
        )
        .await?;
        write_async(&path, &bytes).await?;
    }

    if let Some(loading_bar) = loading_bar {
        emit_loading(loading_bar, 9.0, None).await?;
    }

    log::info!("Downloaded client {} successfully", version_info.id);

    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn download_asset(
    state: &LauncherState,
    name: &String,
    asset: &minecraft::Asset,
    with_legacy: bool,
    force: bool,
) -> crate::Result<()> {
    let hash = &asset.hash;
    let url = format!(
        "{MINECRAFT_RESOURCES_BASE_URL}{sub_hash}/{hash}",
        sub_hash = &hash[..2]
    );
    log::debug!("Downloading asset \"{}\"\n\tfrom {}", name, url);

    let asset_path = state.locations.object_dir(hash);

    let fetch_cell = tokio::sync::OnceCell::<Bytes>::new();

    let res = tokio::try_join! {
        // Download asset
        async  {
            if !asset_path.exists() || force {
                let asset_resource = fetch_cell.get_or_try_init(|| {
                    utils::fetch::fetch_advanced(
                        Method::GET,
                        &url,
                        None,
                        None,
                        None,
                        &state.fetch_semaphore,
                        None
                    )
                }).await?;

                utils::io::write_async(&asset_path, &asset_resource).await?;
            }

            Ok::<(), crate::Error>(())
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
                        None,
                        None,
                        &state.fetch_semaphore,
                        None
                    )
                }).await?;

                utils::io::write_async(&legacy_path, &asset_resource).await?;
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

#[tracing::instrument(skip(state, index))]
pub async fn download_assets(
    state: &LauncherState,
    index: &minecraft::AssetsIndex,
    with_legacy: bool,
    force: bool,
    loading_amount: f64,
    loading_bar: Option<&LoadingBarId>,
) -> crate::Result<()> {
    log::info!("Downloading assets");

    let assets_stream =
        stream::iter(index.objects.iter()).map(Ok::<(&String, &minecraft::Asset), crate::Error>);

    let futures_count = index.objects.len();

    loading_try_for_each_concurrent(
        assets_stream,
        None,
        loading_bar,
        loading_amount,
        futures_count,
        None,
        |(name, asset)| async move {
            log::debug!("Downloading asset \"{}\"", name);
            let res = download_asset(state, name, asset, with_legacy, force).await;
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

#[tracing::instrument(skip_all)]
pub async fn download_java_library(
    state: &LauncherState,
    library: &minecraft::Library,
    force: bool,
) -> crate::Result<()> {
    log::debug!("Downloading java library \"{}\"", &library.name);

    let library_path_part = daedalus::get_path_from_artifact(&library.name)?;
    let library_path = state.locations.libraries_dir().join(&library_path_part);

    if library_path.exists() && !force {
        return Ok::<(), crate::Error>(());
    }

    // Get library by artifact url
    if let Some(minecraft::LibraryDownloads {
        artifact: Some(ref artifact),
        ..
    }) = library.downloads
    {
        if !artifact.url.is_empty() {
            let bytes = utils::fetch::fetch_advanced(
                Method::GET,
                &artifact.url,
                None,
                None,
                None,
                &state.fetch_semaphore,
                None,
            )
            .await?;
            write_async(&library_path, &bytes).await?;
            return Ok::<(), crate::Error>(());
        }
    }
    println!(
        "Library {}, part {}",
        library
            .url
            .as_deref()
            .unwrap_or(MINECRAFT_LIBRARIES_BASE_URL),
        library_path_part
    );

    // Else get library by library.url or default library url
    let url = [
        library
            .url
            .as_deref()
            .unwrap_or(MINECRAFT_LIBRARIES_BASE_URL),
        &library_path_part,
    ]
    .concat();

    println!("Library url {}", url);

    let bytes = utils::fetch::fetch_advanced(
        Method::GET,
        &url,
        None,
        None,
        None,
        &state.fetch_semaphore,
        None,
    )
    .await;

    match bytes {
        Ok(bytes) => {
            write_async(&library_path, &bytes).await?;
            log::debug!("Downloaded java library \"{}\" successfully", &library.name);
            Ok(())
        }
        Err(err) => {
            log::error!("Failed downloading java library \"{}\"", &library.name,);
            Err(err)
        }
    }
}

#[tracing::instrument(skip_all)]
pub async fn download_native_library_files(
    state: &LauncherState,
    library: &minecraft::Library,
    version_info: &minecraft::VersionInfo,
    java_arch: &str,
    force: bool,
) -> crate::Result<()> {
    use crate::utils::platform::OsExt;
    use minecraft::Os;

    log::debug!("Downloading native library \"{}\"", &library.name);

    if let Some((os_key, classifiers)) = None.or_else(|| {
        Some((
            library.natives.as_ref()?.get(&Os::native_arch(java_arch))?,
            library.downloads.as_ref()?.classifiers.as_ref()?,
        ))
    }) {
        let parsed_key = os_key.replace("${arch}", crate::utils::platform::ARCH_WIDTH);

        if let Some(native) = classifiers.get(&parsed_key) {
            let bytes = utils::fetch::fetch_advanced(
                Method::GET,
                &native.url,
                None,
                None,
                None,
                &state.fetch_semaphore,
                None,
            )
            .await?;
            let reader = std::io::Cursor::new(&bytes);

            if let Ok(mut archive) = zip::ZipArchive::new(reader) {
                match archive.extract(state.locations.version_natives_dir(&version_info.id)) {
                    Ok(_) => log::debug!("Extracted native library {}", &library.name),
                    Err(err) => {
                        log::error!(
                            "Failed extracting native library {}. err: {}",
                            &library.name,
                            err
                        )
                    }
                }
            } else {
                log::error!("Failed extracting native library {}", &library.name)
            }
        }
    }

    log::debug!(
        "Downloaded native library \"{}\" successfully",
        &library.name
    );

    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn download_library(
    state: &LauncherState,
    library: &minecraft::Library,
    version_info: &minecraft::VersionInfo,
    java_arch: &str,
    force: bool,
    minecraft_updated: bool,
) -> crate::Result<()> {
    if let Some(rules) = &library.rules {
        if !parse_rules(rules, java_arch, minecraft_updated) {
            return Ok(());
        }
    }

    if !library.downloadable {
        return Ok(());
    }

    tokio::try_join! {
        download_java_library(state, library, force),
        download_native_library_files(state, library, version_info, java_arch, force)
    }?;

    Ok(())
}

// TODO: reduce arguments count
#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip(state, libraries))]
pub async fn download_libraries(
    state: &LauncherState,
    libraries: &[minecraft::Library],
    version_info: &minecraft::VersionInfo,
    java_arch: &str,
    force: bool,
    minecraft_updated: bool,
    loading_amount: f64,
    loading_bar: Option<&LoadingBarId>,
) -> crate::Result<()> {
    log::info!("Downloading libraries for {}", version_info.id);

    tokio::try_join! {
        tokio::fs::create_dir_all(state.locations.libraries_dir()),
        tokio::fs::create_dir_all(state.locations.version_natives_dir(&version_info.id)),
    }?;

    let libraries_stream =
        stream::iter(libraries.iter()).map(Ok::<&minecraft::Library, crate::Error>);

    let futures_count = libraries.len();

    loading_try_for_each_concurrent(
        libraries_stream,
        None,
        loading_bar,
        loading_amount,
        futures_count,
        None,
        |library| async move {
            download_library(
                state,
                library,
                version_info,
                java_arch,
                force,
                minecraft_updated,
            )
            .await
        },
    )
    .await?;

    log::info!("Downloaded libraries for {} successfully", version_info.id);

    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn download_version_manifest(
    state: &LauncherState,
    force: bool,
) -> crate::Result<minecraft::VersionManifest> {
    let path = state.locations.versions_dir().join("manifest.json");

    let res = if path.exists() && !force {
        utils::io::read_json_async(path).await
    } else {
        let version_manifest = utils::fetch::fetch_json(
            Method::GET,
            minecraft::VERSION_MANIFEST_URL,
            None,
            None,
            None,
            &state.fetch_semaphore,
        )
        .await?;

        utils::io::write_json_async(&path, &version_manifest).await?;

        Ok(version_manifest)
    }?;

    Ok(res)
}

#[tracing::instrument(skip_all)]
pub async fn download_loaders_manifests(
    state: &LauncherState,
    loader: &str,
    force: bool,
) -> crate::Result<modded::Manifest> {
    let path = state
        .locations
        .cache_dir()
        .join("mod_loaders")
        .join(format!("{loader}-manifest.json"));

    let loaders_manifest_url = format!("{META_URL}{}/v0/manifest.json", loader);

    let res = if path.exists() && !force {
        utils::io::read_json_async(path).await
    } else {
        let loaders_manifest = utils::fetch::fetch_json::<modded::Manifest>(
            Method::GET,
            &loaders_manifest_url,
            None,
            None,
            None,
            &state.fetch_semaphore,
        )
        .await?;

        utils::io::write_json_async(path, &loaders_manifest).await?;

        Ok(loaders_manifest)
    }?;

    Ok(res)

    // let keys = [
    //     ModLoader::Forge,
    //     ModLoader::Fabric,
    //     ModLoader::Quilt,
    //     ModLoader::NeoForge,
    // ];

    // let fetch_urls = keys
    //     .iter()
    //     .map(|x| (x, format!("{META_URL}{}/v0/manifest.json", x.as_meta_str())))
    //     .collect::<Vec<_>>();

    // let res = futures::future::try_join_all(fetch_urls.iter().map(|(_, url)| {
    //     fetch_json::<modded::Manifest>(Method::GET, url, None, None, None, &state.fetch_semaphore)
    // }))
    // .await?
    // .into_iter()
    // .enumerate()
    // .map(|(index, metadata)| ModLoaderManifest {
    //     loader: *fetch_urls[index].0,
    //     manifest: metadata,
    // })
    // .collect();

    // Ok(res)
}
