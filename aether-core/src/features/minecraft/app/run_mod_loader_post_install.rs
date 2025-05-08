use std::{path::PathBuf, sync::Arc};

use daedalus::minecraft::VersionInfo;

use crate::features::{
    events::{EventEmitter, ProgressBarId, ProgressBarStorage, ProgressService},
    instance::Instance,
    java::Java,
    minecraft::{ForgeProcessor, ModLoader, ModLoaderProcessor},
    settings::LocationInfo,
};

pub async fn run_mod_loader_post_install<E: EventEmitter, PBS: ProgressBarStorage>(
    progress_service: Arc<ProgressService<E, PBS>>,
    location_info: Arc<LocationInfo>,
    instance: &Instance,
    version_jar: String,
    instance_path: &PathBuf,
    version_info: &mut VersionInfo,
    java_version: &Java,
    loading_bar: Option<&ProgressBarId>,
) -> crate::Result<()> {
    match instance.loader {
        ModLoader::Vanilla => Ok(()),
        ModLoader::Forge => {
            ForgeProcessor::new(progress_service, location_info)
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
