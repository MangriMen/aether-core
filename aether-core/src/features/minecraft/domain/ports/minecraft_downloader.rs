use async_trait::async_trait;

use crate::features::{events::ProgressBarId, minecraft::MinecraftError};

#[async_trait]
pub trait MinecraftDownloader: Send + Sync {
    async fn download_minecraft(
        &self,
        version_info: &daedalus::minecraft::VersionInfo,
        java_arch: &str,
        force: bool,
        minecraft_updated: bool,
        loading_bar: Option<&ProgressBarId>,
    ) -> Result<(), MinecraftError>;

    async fn download_version_info(
        &self,
        version: &daedalus::minecraft::Version,
        loader: Option<&daedalus::modded::LoaderVersion>,
        force: Option<bool>,
        loading_bar: Option<&ProgressBarId>,
    ) -> Result<daedalus::minecraft::VersionInfo, MinecraftError>;
}
