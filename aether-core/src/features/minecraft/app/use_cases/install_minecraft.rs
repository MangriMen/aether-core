use std::{path::Path, sync::Arc};

use async_trait::async_trait;

use crate::{
    core::LauncherState,
    features::{
        events::{
            EventEmitter, ProgressBarId, ProgressBarStorage, ProgressEventType, ProgressService,
        },
        instance::{InstanceInstallStage, InstanceStorage, InstanceStorageExtensions},
        minecraft::{
            self, GetVersionManifestUseCase, LoaderVersionResolver, ModLoader, ReadMetadataStorage,
        },
        settings::LocationInfo,
    },
    shared::domain::{AsyncUseCaseWithError, AsyncUseCaseWithInputAndError},
};

pub struct InstallMinecraftUseCase<
    E: EventEmitter,
    PS: ProgressBarStorage,
    IS: InstanceStorage,
    MS: ReadMetadataStorage,
> {
    progress_service: Arc<ProgressService<E, PS>>,
    instance_storage: Arc<IS>,
    loader_version_resolver: Arc<LoaderVersionResolver<MS>>,
    get_version_manifest_use_case: Arc<GetVersionManifestUseCase<MS>>,
    location_info: Arc<LocationInfo>,
}

impl<E: EventEmitter, PS: ProgressBarStorage, IS: InstanceStorage, MS: ReadMetadataStorage>
    InstallMinecraftUseCase<E, PS, IS, MS>
{
    pub fn new(
        progress_service: Arc<ProgressService<E, PS>>,
        instance_storage: Arc<IS>,
        loader_version_resolver: Arc<LoaderVersionResolver<MS>>,
        get_version_manifest_use_case: Arc<GetVersionManifestUseCase<MS>>,
        location_info: Arc<LocationInfo>,
    ) -> Self {
        Self {
            progress_service,
            instance_storage,
            loader_version_resolver,
            get_version_manifest_use_case,
            location_info,
        }
    }
}

#[async_trait]
impl<E: EventEmitter, PS: ProgressBarStorage, IS: InstanceStorage, MS: ReadMetadataStorage>
    AsyncUseCaseWithInputAndError for InstallMinecraftUseCase<E, PS, IS, MS>
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
                minecraft::resolve_minecraft_version(&instance.game_version, version_manifest)?;

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

            let state = LauncherState::get().await?;

            let mut version_info = minecraft::download_version_info(
                &state,
                &version,
                loader_version.as_ref(),
                Some(force),
                Some(&loading_bar),
            )
            .await?;

            let java = if let Some(java_path) = instance.java_path.as_ref() {
                crate::features::java::get_java_from_path(Path::new(java_path)).await
            } else {
                let compatible_java_version = minecraft::get_compatible_java_version(&version_info);

                let java = crate::api::java::get(compatible_java_version).await;

                match java {
                    Ok(java) => Ok(java),
                    Err(_) => crate::api::java::install(compatible_java_version).await,
                }
            }?;

            minecraft::download_minecraft(
                &state,
                &version_info,
                &java.architecture,
                force,
                minecraft_updated,
                Some(&loading_bar),
            )
            .await?;

            minecraft::run_mod_loader_post_install(
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
