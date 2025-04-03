use std::path::PathBuf;

// Install JRE
#[tracing::instrument]
pub async fn install(version: u32) -> crate::Result<PathBuf> {
    crate::features::java::install_jre(version).await
}
