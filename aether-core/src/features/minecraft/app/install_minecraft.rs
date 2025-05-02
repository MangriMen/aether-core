use std::{path::Path, sync::Arc};

use crate::{
    api,
    core::LauncherState,
    features::{
        events::{emit_loading, init_or_edit_loading, LoadingBarId, LoadingBarType},
        instance::{Instance, InstanceInstallStage, InstanceStorage, InstanceStorageExtensions},
        minecraft::{self, LoaderVersionResolver, ModLoader, ReadMetadataStorage},
    },
};

#[tracing::instrument(skip(instance_storage, loader_version_resolver))]
pub async fn install_minecraft<IS, MS>(
    instance_storage: Arc<IS>,
    loader_version_resolver: &LoaderVersionResolver<MS>,
    instance: &Instance,
    loading_bar: Option<LoadingBarId>,
    force: bool,
) -> crate::Result<()>
where
    IS: InstanceStorage + Send + Sync,
    MS: ReadMetadataStorage,
{
    log::info!(
        "Installing instance: \"{}\" (minecraft: \"{}\", modloader: \"{}\")",
        instance.name,
        instance.game_version,
        instance.loader_version.clone().unwrap_or_default()
    );

    let loading_bar = init_or_edit_loading(
        loading_bar,
        LoadingBarType::MinecraftDownload {
            instance_path: instance.id.clone(),
            instance_name: instance.name.clone(),
        },
        100.0,
        "Downloading Minecraft",
    )
    .await?;

    instance_storage
        .upsert_with(&instance.id, |instance| {
            instance.install_stage = InstanceInstallStage::Installing;
            Ok(())
        })
        .await?;

    let result = async {
        let state = LauncherState::get().await?;

        let instance_path = state.locations.instance_dir(&instance.id);

        let version_manifest = api::metadata::get_version_manifest().await?;

        let (version, minecraft_updated) =
            minecraft::resolve_minecraft_version(&instance.game_version, version_manifest)?;

        let loader_version = if instance.loader == ModLoader::Vanilla {
            None
        } else {
            let loader_version = loader_version_resolver
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
                    let stable_loader_version = loader_version_resolver
                        .resolve(
                            &instance.game_version,
                            &instance.loader,
                            &Some("stable".to_string()),
                        )
                        .await?;

                    instance_storage
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
            instance,
            version_jar,
            &instance_path,
            &mut version_info,
            &java,
            Some(&loading_bar),
        )
        .await?;

        instance_storage
            .upsert_with(&instance.id, |instance| {
                instance.install_stage = InstanceInstallStage::Installed;
                Ok(())
            })
            .await?;

        emit_loading(&loading_bar, 1.000_000_000_01, Some("Finished installing")).await?;

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
            instance_storage
                .upsert_with(&instance.id, |instance| {
                    instance.install_stage = InstanceInstallStage::NotInstalled;
                    Ok(())
                })
                .await?;
            Err(e)
        }
    }
}
