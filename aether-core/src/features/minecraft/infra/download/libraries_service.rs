use std::sync::Arc;

use futures::StreamExt;

use crate::{
    features::{
        events::{
            try_for_each_concurrent_with_progress, ProgressConfig, ProgressConfigWithMessage,
            ProgressService,
        },
        minecraft::{parse_rules, MinecraftError},
        settings::LocationInfo,
    },
    libs::request_client::{Request, RequestClient},
    shared::{write_async, IoError},
};

const MINECRAFT_LIBRARIES_BASE_URL: &str = "https://libraries.minecraft.net/";

pub struct LibrariesService<RC: RequestClient, PS: ProgressService> {
    progress_service: Arc<PS>,
    request_client: Arc<RC>,
    location_info: Arc<LocationInfo>,
}

impl<RC: RequestClient, PS: ProgressService> LibrariesService<RC, PS> {
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

    pub async fn download_libraries(
        &self,
        libraries: &[daedalus::minecraft::Library],
        version_info: &daedalus::minecraft::VersionInfo,
        java_arch: &str,
        force: bool,
        minecraft_updated: bool,
        progress_config: Option<&ProgressConfig<'_>>,
    ) -> Result<(), MinecraftError> {
        log::info!("Downloading libraries for {}", version_info.id);

        tokio::try_join! {
            tokio::fs::create_dir_all(self.location_info.libraries_dir()),
            tokio::fs::create_dir_all(self.location_info.version_natives_dir(&version_info.id)),
        }
        .map_err(IoError::from)?;

        let libraries_stream = futures::stream::iter(libraries.iter())
            .map(Ok::<&daedalus::minecraft::Library, MinecraftError>);

        let futures_count = libraries.len();

        let progress_config = progress_config
            .as_ref()
            .map(|config| ProgressConfigWithMessage {
                progress_bar_id: config.progress_bar_id,
                total_progress: config.total_progress,
                progress_message: None,
            });

        try_for_each_concurrent_with_progress(
            self.progress_service.clone(),
            libraries_stream,
            None,
            futures_count,
            progress_config.as_ref(),
            |library| async move {
                self.download_library(library, version_info, java_arch, force, minecraft_updated)
                    .await
            },
        )
        .await?;

        log::info!("Downloaded libraries for {} successfully", version_info.id);

        Ok(())
    }

    pub async fn download_library(
        &self,
        library: &daedalus::minecraft::Library,
        version_info: &daedalus::minecraft::VersionInfo,
        java_arch: &str,
        force: bool,
        minecraft_updated: bool,
    ) -> Result<(), MinecraftError> {
        if let Some(rules) = &library.rules {
            if !parse_rules(rules, java_arch, minecraft_updated) {
                return Ok(());
            }
        }

        if !library.downloadable {
            return Ok(());
        }

        tokio::try_join! {
            self.download_java_library( library, force),
            self.download_native_library_files( library, version_info, java_arch, force)
        }?;

        Ok(())
    }

    pub async fn download_java_library(
        &self,
        library: &daedalus::minecraft::Library,
        force: bool,
    ) -> Result<(), MinecraftError> {
        log::debug!("Downloading java library \"{}\"", &library.name);

        let library_path_part = daedalus::get_path_from_artifact(&library.name)?;
        let library_path = self.location_info.libraries_dir().join(&library_path_part);

        if library_path.exists() && !force {
            return Ok(());
        }

        // Get library by artifact url
        if let Some(daedalus::minecraft::LibraryDownloads {
            artifact: Some(ref artifact),
            ..
        }) = library.downloads
        {
            if !artifact.url.is_empty() {
                let bytes = self
                    .request_client
                    .fetch_bytes(Request::get(&artifact.url))
                    .await
                    .map_err(|err| {
                        IoError::IOError(std::io::Error::new(
                            std::io::ErrorKind::NetworkUnreachable,
                            err,
                        ))
                    })?;
                write_async(&library_path, &bytes).await?;
                return Ok(());
            }
        }
        log::debug!(
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

        log::debug!("Library url {}", url);

        let bytes = self.request_client.fetch_bytes(Request::get(&url)).await;

        match bytes {
            Ok(bytes) => {
                write_async(&library_path, &bytes).await?;
                log::debug!("Downloaded java library \"{}\" successfully", &library.name);
                Ok(())
            }
            Err(err) => {
                log::error!("Failed downloading java library \"{}\"", &library.name,);
                Ok(Err(IoError::IOError(std::io::Error::new(
                    std::io::ErrorKind::NetworkUnreachable,
                    err,
                )))?)
            }
        }
    }

    pub async fn download_native_library_files(
        &self,
        library: &daedalus::minecraft::Library,
        version_info: &daedalus::minecraft::VersionInfo,
        java_arch: &str,
        _force: bool,
    ) -> Result<(), MinecraftError> {
        use crate::shared::OsExt;
        use daedalus::minecraft::Os;

        log::debug!("Downloading native library \"{}\"", &library.name);

        if let Some((os_key, classifiers)) = None.or_else(|| {
            Some((
                library.natives.as_ref()?.get(&Os::native_arch(java_arch))?,
                library.downloads.as_ref()?.classifiers.as_ref()?,
            ))
        }) {
            let parsed_key = os_key.replace("${arch}", crate::shared::ARCH_WIDTH);

            if let Some(native) = classifiers.get(&parsed_key) {
                let bytes = self
                    .request_client
                    .fetch_bytes(Request::get(&native.url))
                    .await
                    .map_err(|err| {
                        IoError::IOError(std::io::Error::new(
                            std::io::ErrorKind::NetworkUnreachable,
                            err,
                        ))
                    })?;
                let reader = std::io::Cursor::new(&bytes);

                if let Ok(mut archive) = zip::ZipArchive::new(reader) {
                    match archive.extract(self.location_info.version_natives_dir(&version_info.id))
                    {
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
}
