use std::path::Path;

use async_trait::async_trait;
use daedalus::minecraft::VersionInfo;

use crate::features::{events::ProgressBarId, instance::Instance, java::Java};

#[async_trait]
pub trait ModLoaderProcessor {
    async fn run(
        &self,
        instance: &Instance,
        version_jar: String,
        instance_path: &Path,
        version_info: &mut VersionInfo,
        java_version: &Java,
        loading_bar: Option<&ProgressBarId>,
    ) -> crate::Result<()>;
}
