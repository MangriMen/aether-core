use std::{path::Path, sync::Arc};

use async_trait::async_trait;
use daedalus::minecraft::VersionInfo;

use crate::{
    features::{
        events::{ProgressBarId, ProgressEventType, ProgressService},
        instance::{Instance, InstanceInstallStage, InstanceStorage, InstanceStorageExtensions},
        java::Java,
        minecraft::{
            get_compatible_java_version, resolve_minecraft_version, ForgeProcessor,
            GetVersionManifestUseCase, LoaderVersionResolver, MinecraftDownloader, ModLoader,
            ModLoaderProcessor, ReadMetadataStorage,
        },
        settings::LocationInfo,
    },
    shared::domain::{AsyncUseCaseWithError, AsyncUseCaseWithInputAndError},
};

pub struct InstallMinecraftUseCase<
    IS: InstanceStorage,
    MS: ReadMetadataStorage,
    MD: MinecraftDownloader,
    PS: ProgressService,
> {
    progress_service: Arc<PS>,
    instance_storage: Arc<IS>,
    loader_version_resolver: Arc<LoaderVersionResolver<MS>>,
    get_version_manifest_use_case: Arc<GetVersionManifestUseCase<MS>>,
    location_info: Arc<LocationInfo>,
    minecraft_download_service: MD,
}

impl<
        IS: InstanceStorage,
        MS: ReadMetadataStorage,
        MD: MinecraftDownloader,
        PS: ProgressService,
    > InstallMinecraftUseCase<IS, MS, MD, PS>
{
    pub fn new(
        progress_service: Arc<PS>,
        instance_storage: Arc<IS>,
        loader_version_resolver: Arc<LoaderVersionResolver<MS>>,
        get_version_manifest_use_case: Arc<GetVersionManifestUseCase<MS>>,
        location_info: Arc<LocationInfo>,
        minecraft_download_service: MD,
    ) -> Self {
        Self {
            progress_service,
            instance_storage,
            loader_version_resolver,
            get_version_manifest_use_case,
            location_info,
            minecraft_download_service,
        }
    }

    async fn run_mod_loader_post_install(
        &self,
        instance: &Instance,
        version_jar: String,
        instance_path: &Path,
        version_info: &mut VersionInfo,
        java_version: &Java,
        loading_bar: Option<&ProgressBarId>,
    ) -> crate::Result<()> {
        match instance.loader {
            ModLoader::Vanilla => Ok(()),
            ModLoader::Forge => {
                ForgeProcessor::new(self.progress_service.clone(), self.location_info.clone())
                    .run(
                        instance,
                        version_jar,
                        instance_path,
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
}

#[async_trait]
impl<
        IS: InstanceStorage,
        MS: ReadMetadataStorage,
        MD: MinecraftDownloader,
        PS: ProgressService,
    > AsyncUseCaseWithInputAndError for InstallMinecraftUseCase<IS, MS, MD, PS>
{
    type Input = (String, Option<ProgressBarId>, bool);
    type Output = ();
    type Error = crate::Error;

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let (instance_id, loading_bar, force) = input;

        let instance = self.instance_storage.get(&instance_id).await?;

        log::info!(
            "Installing instance: \"{}\" (minecraft: \"{}\", modloader: \"{}\")",
            instance.name,
            instance.game_version,
            instance.loader_version.clone().unwrap_or_default()
        );

        let loading_bar = self.progress_service.init_or_edit_progress(
            loading_bar,
            ProgressEventType::MinecraftDownload {
                instance_id: instance.id.clone(),
                instance_name: instance.name.clone(),
            },
            100.0,
            "Downloading Minecraft".to_string(),
        )?;

        self.instance_storage
            .upsert_with(&instance.id, |instance| {
                instance.install_stage = InstanceInstallStage::Installing;
                Ok(())
            })
            .await?;

        let result = async {
            let instance_path = self.location_info.instance_dir(&instance.id);

            let version_manifest = self.get_version_manifest_use_case.execute().await?;

            let (version, minecraft_updated) =
                resolve_minecraft_version(&instance.game_version, version_manifest)?;

            let loader_version = if instance.loader == ModLoader::Vanilla {
                None
            } else {
                let loader_version = self
                    .loader_version_resolver
                    .resolve(
                        &instance.game_version,
                        &instance.loader,
                        &instance.loader_version,
                    )
                    .await?;

                match loader_version {
                    Some(loader_version) => Some(loader_version),
                    None => {
                        // If no loader version is selected, try to select the stable version!
                        let stable_loader_version = self
                            .loader_version_resolver
                            .resolve(
                                &instance.game_version,
                                &instance.loader,
                                &Some("stable".to_string()),
                            )
                            .await?;

                        self.instance_storage
                            .upsert_with(&instance.id, |instance| {
                                instance.loader_version =
                                    stable_loader_version.clone().map(|x| x.id.clone());
                                Ok(())
                            })
                            .await?;

                        stable_loader_version
                    }
                }
            };

            let version_jar = loader_version.as_ref().map_or(version.id.clone(), |it| {
                format!("{}-{}", version.id.clone(), it.id.clone())
            });

            let mut version_info = self
                .minecraft_download_service
                .download_version_info(
                    &version,
                    loader_version.as_ref(),
                    Some(force),
                    Some(&loading_bar),
                )
                .await?;

            let java = if let Some(java_path) = instance.java_path.as_ref() {
                crate::features::java::get_java_from_path(Path::new(java_path)).await
            } else {
                let compatible_java_version = get_compatible_java_version(&version_info);

                let java = crate::api::java::get(compatible_java_version).await;

                match java {
                    Ok(java) => Ok(java),
                    Err(_) => crate::api::java::install(compatible_java_version).await,
                }
            }?;

            self.minecraft_download_service
                .download_minecraft(
                    &version_info,
                    &java.architecture,
                    force,
                    minecraft_updated,
                    Some(&loading_bar),
                )
                .await?;

            self.run_mod_loader_post_install(
                &instance,
                version_jar,
                &instance_path,
                &mut version_info,
                &java,
                Some(&loading_bar),
            )
            .await?;

            self.instance_storage
                .upsert_with(&instance.id, |instance| {
                    instance.install_stage = InstanceInstallStage::Installed;
                    Ok(())
                })
                .await?;

            self.progress_service.emit_progress(
                &loading_bar,
                1.000_000_000_01,
                Some("Finished installing"),
            )?;

            Ok(())
        }
        .await;

        match result {
            Ok(_) => {
                log::info!(
                    "Installed instance: \"{}\" (minecraft: \"{}\", modloader: \"{}\")",
                    instance.name,
                    instance.game_version,
                    instance.loader_version.clone().unwrap_or_default()
                );
                Ok(())
            }
            Err(e) => {
                self.instance_storage
                    .upsert_with(&instance.id, |instance| {
                        instance.install_stage = InstanceInstallStage::NotInstalled;
                        Ok(())
                    })
                    .await?;
                Err(e)
            }
        }
    }
}
