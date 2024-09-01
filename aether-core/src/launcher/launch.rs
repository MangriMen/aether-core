use anyhow::Ok;
use daedalus::modded;

use crate::state::{Instance, InstanceInstallStage, LauncherState};

use super::{download_minecraft, download_version_info, download_version_manifest};

#[tracing::instrument]
pub async fn install_minecraft(instance: &Instance, repairing: bool) -> anyhow::Result<()> {
    let state = LauncherState::get().await?;

    // For mod loader processing
    // let instance_path = instance.get_full_path().await?;

    let version_manifest = download_version_manifest(&state, false).await?;

    let version_index = version_manifest
        .versions
        .iter()
        .position(|version| version.id == instance.game_version)
        .ok_or(anyhow::Error::msg(format!(
            "Invalid game version: {}",
            instance.game_version
        )))?;

    let version = &version_manifest.versions[version_index];

    let minecraft_updated = version_index
        <= version_manifest
            .versions
            .iter()
            .position(|version| version.id == "22w16a")
            .unwrap_or(0);

    // TODO: add mod loader support
    // let mut mod_loader_version =

    let loader_version: Option<modded::LoaderVersion> = None;

    // For mod loader processing
    // let version_jar = loader_version.as_ref().map_or(version.id.clone(), |it| {
    //     format!("{}-{}", version.id.clone(), it.id.clone())
    // });

    let version_info =
        download_version_info(&state, version, loader_version.as_ref(), None).await?;

    let java_version = instance
        .get_java_version_from_profile(&version_info)
        .await?
        .ok_or_else(|| anyhow::Error::msg("Missing correct java installation"))?;

    let java_version = crate::api::jre::check_jre(java_version.path.clone().into())
        .await?
        .ok_or_else(|| {
            anyhow::Error::msg(format!(
                "Java path invalid or non-functional: {}",
                java_version.path
            ))
        })?;

    download_minecraft(
        &state,
        &version_info,
        &java_version.architecture,
        repairing,
        minecraft_updated,
    )
    .await?;

    // @ process mod loader

    Ok(())
}

#[tracing::instrument]
pub async fn launch_minecraft(instance: &Instance) -> anyhow::Result<()> {
    if instance.install_stage != InstanceInstallStage::Installed {
        install_minecraft(instance, false).await?;
    }
    // if not installed -> install_minecraft

    // state

    // instance path

    // minecraft versions manifest

    // minecraft version

    // mod loader version

    // jar name

    // version info

    // java version

    // client path

    // env args

    // check existing

    // natives dir?

    // create command
    //  jvm args
    //  version main class
    //  minecraft arguments
    //  current dir (instance path)

    //  remove _JAVA_OPTIONS

    //  env args

    // options.txt override

    // authentication credentials

    // minimize launcher

    // run process

    Ok(())
}
