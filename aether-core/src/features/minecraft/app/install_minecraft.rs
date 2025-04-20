use crate::{
    api,
    core::LauncherState,
    features::{
        events::{
            emit::{emit_loading, init_or_edit_loading},
            LoadingBarId, LoadingBarType,
        },
        instance::{Instance, InstanceInstallStage},
        minecraft::{self, ModLoader},
    },
};

#[tracing::instrument]
pub async fn install_minecraft(
    instance: &Instance,
    loading_bar: Option<LoadingBarId>,
    force: bool,
) -> crate::Result<()> {
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

    Instance::edit(&instance.id, |instance| {
        instance.install_stage = InstanceInstallStage::Installing;
        async { Ok(()) }
    })
    .await?;

    let result = async {
        let state = LauncherState::get().await?;

        let instance_path = Instance::get_full_path(&instance.id).await?;

        let version_manifest = api::metadata::get_version_manifest().await?;

        let (version, minecraft_updated) =
            minecraft::resolve_minecraft_version(&instance.game_version, version_manifest)?;

        let mut loader_version = Instance::get_loader_version(
            &instance.game_version,
            instance.loader,
            instance.loader_version.as_deref(),
        )
        .await?;

        // If no loader version is selected, try to select the stable version!
        if instance.loader != ModLoader::Vanilla && loader_version.is_none() {
            loader_version = Instance::get_loader_version(
                &instance.game_version,
                instance.loader,
                Some("stable"),
            )
            .await?;

            let loader_version_id = loader_version.clone();

            Instance::edit(&instance.id, |instance| {
                instance.loader_version = loader_version_id.clone().map(|x| x.id.clone());
                async { Ok(()) }
            })
            .await?;
        }

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

        let java = if let Some(java) = Instance::get_java(instance).await.transpose() {
            java
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

        Instance::edit(&instance.id, |prof| {
            prof.install_stage = InstanceInstallStage::Installed;
            async { Ok(()) }
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
            Instance::edit(&instance.id, |prof| {
                prof.install_stage = InstanceInstallStage::NotInstalled;
                async { Ok(()) }
            })
            .await?;

            Err(e)
        }
    }
}
