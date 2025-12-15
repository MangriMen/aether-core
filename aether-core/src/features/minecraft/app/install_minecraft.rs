use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use crate::{
    features::{
        events::{ProgressBarId, ProgressService},
        java::{GetJavaUseCase, InstallJavaUseCase, Java, JavaInstallationService, JavaStorage},
        minecraft::{
            get_compatible_java_version, resolve_minecraft_version, ForgeProcessor,
            GetVersionManifestUseCase, LoaderVersionPreference, LoaderVersionResolver,
            MinecraftDownloader, MinecraftError, ModLoader, ModLoaderProcessor,
            ReadMetadataStorage,
        },
        settings::LocationInfo,
    },
    libs::request_client::RequestClient,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct InstallMinecraftParams {
    pub game_version: String,
    pub loader: ModLoader,
    pub loader_version: Option<LoaderVersionPreference>,
    pub install_dir: PathBuf,
    pub java_path: Option<String>,
}

pub struct InstallMinecraftUseCase<
    MS: ReadMetadataStorage,
    MD: MinecraftDownloader,
    PS: ProgressService,
    JIS: JavaInstallationService,
    JS: JavaStorage,
    RC: RequestClient,
> {
    progress_service: Arc<PS>,
    loader_version_resolver: Arc<LoaderVersionResolver<MS>>,
    get_version_manifest_use_case: Arc<GetVersionManifestUseCase<MS>>,
    location_info: Arc<LocationInfo>,
    minecraft_download_service: MD,
    java_installation_service: JIS,
    get_java_use_case: Arc<GetJavaUseCase<JS, JIS>>,
    install_java_use_case: Arc<InstallJavaUseCase<JS, JIS, PS, RC>>,
}

impl<
        MS: ReadMetadataStorage,
        MD: MinecraftDownloader,
        PS: ProgressService,
        JIS: JavaInstallationService,
        JS: JavaStorage,
        RC: RequestClient,
    > InstallMinecraftUseCase<MS, MD, PS, JIS, JS, RC>
{
    // TODO: try to decrease arguments count
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        progress_service: Arc<PS>,
        loader_version_resolver: Arc<LoaderVersionResolver<MS>>,
        get_version_manifest_use_case: Arc<GetVersionManifestUseCase<MS>>,
        location_info: Arc<LocationInfo>,
        minecraft_download_service: MD,
        java_installation_service: JIS,
        get_java_use_case: Arc<GetJavaUseCase<JS, JIS>>,
        install_java_use_case: Arc<InstallJavaUseCase<JS, JIS, PS, RC>>,
    ) -> Self {
        Self {
            progress_service,
            loader_version_resolver,
            get_version_manifest_use_case,
            location_info,
            minecraft_download_service,
            java_installation_service,
            get_java_use_case,
            install_java_use_case,
        }
    }

    // TODO: try to decrease arguments count
    #[allow(clippy::too_many_arguments)]
    async fn run_mod_loader_post_install(
        &self,
        game_version: String,
        loader: ModLoader,
        version_jar: String,
        minecraft_dir: &Path,
        version_info: &mut daedalus::minecraft::VersionInfo,
        java_version: &Java,
        loading_bar: Option<&ProgressBarId>,
    ) -> Result<(), MinecraftError> {
        match loader {
            ModLoader::Vanilla => Ok(()),
            ModLoader::Forge => {
                ForgeProcessor::new(self.progress_service.clone(), self.location_info.clone())
                    .run(
                        game_version,
                        version_jar,
                        minecraft_dir,
                        version_info,
                        java_version,
                        loading_bar,
                    )
                    .await
            }
            ModLoader::Fabric => Ok(()),
            ModLoader::Quilt => Ok(()),
            ModLoader::NeoForge => Ok(()),
        }
    }

    pub async fn execute(
        &self,
        install_minecraft_params: InstallMinecraftParams,
        loading_bar: Option<&ProgressBarId>,
        force: bool,
    ) -> Result<(), MinecraftError> {
        let InstallMinecraftParams {
            game_version,
            loader,
            loader_version,
            install_dir,
            java_path,
        } = install_minecraft_params;

        let version_manifest = self.get_version_manifest_use_case.execute().await?;

        let (version, minecraft_updated) =
            resolve_minecraft_version(&game_version, version_manifest)?;

        let loader_version = self
            .loader_version_resolver
            .resolve(&game_version, &loader, loader_version.as_ref())
            .await?;

        let version_jar = loader_version.as_ref().map_or(
            version.id.clone(), // For Vanilla take pure version
            |it| format!("{}-{}", version.id.clone(), it.id.clone()),
        );

        let mut version_info = self
            .minecraft_download_service
            .download_version_info(&version, loader_version.as_ref(), Some(force), loading_bar)
            .await?;

        let java = if let Some(java_path) = java_path.as_ref() {
            self.java_installation_service
                .locate_java(Path::new(java_path))
                .await
                .map_err(|_| MinecraftError::JavaNotFound {
                    path: PathBuf::from(java_path),
                })
        } else {
            let compatible_java_version = get_compatible_java_version(&version_info);

            let java = self
                .get_java_use_case
                .execute(compatible_java_version)
                .await;

            match java {
                Ok(java) => Ok(java),
                Err(_) => self
                    .install_java_use_case
                    .execute(compatible_java_version)
                    .await
                    .map_err(|err| MinecraftError::JavaInstallationFailed(err.to_string())),
            }
        }?;

        self.minecraft_download_service
            .download_minecraft(
                &version_info,
                &java.architecture,
                force,
                minecraft_updated,
                loading_bar,
            )
            .await?;

        self.run_mod_loader_post_install(
            game_version,
            loader,
            version_jar,
            &install_dir,
            &mut version_info,
            &java,
            loading_bar,
        )
        .await?;

        Ok(())
    }
}
