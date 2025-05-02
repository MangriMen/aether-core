use std::path::PathBuf;

use daedalus::minecraft::VersionInfo;

use crate::features::{
    events::LoadingBarId,
    instance::Instance,
    java::Java,
    minecraft::{run_forge_processors, ModLoader},
};

pub async fn run_mod_loader_post_install(
    instance: &Instance,
    version_jar: String,
    instance_path: &PathBuf,
    version_info: &mut VersionInfo,
    java_version: &Java,
    loading_bar: Option<&LoadingBarId>,
) -> crate::Result<()> {
    match instance.loader {
        ModLoader::Vanilla => Ok(()),
        ModLoader::Forge => {
            run_forge_processors(
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
