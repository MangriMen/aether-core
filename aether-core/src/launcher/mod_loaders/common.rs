use std::path::PathBuf;

use daedalus::minecraft::VersionInfo;

use crate::{
    event::LoadingBarId,
    state::{Instance, Java},
};

use super::process_forge_processors;

pub async fn mod_loader_post_install(
    instance: &Instance,
    version_jar: String,
    instance_path: &PathBuf,
    version_info: &mut VersionInfo,
    java_version: &Java,
    loading_bar: Option<&LoadingBarId>,
) -> crate::Result<()> {
    match instance.loader {
        crate::state::ModLoader::Vanilla => Ok(()),
        crate::state::ModLoader::Forge => {
            process_forge_processors(
                instance,
                version_jar,
                instance_path,
                version_info,
                java_version,
                loading_bar,
            )
            .await
        }
        crate::state::ModLoader::Fabric => Ok(()),
        crate::state::ModLoader::Quilt => Ok(()),
        crate::state::ModLoader::NeoForge => Ok(()),
    }
}
