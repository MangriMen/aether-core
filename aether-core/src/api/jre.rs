use std::path::PathBuf;

use crate::{core::LauncherState, features::java};

#[tracing::instrument]
pub async fn install(version: u32) -> crate::Result<PathBuf> {
    let state = LauncherState::get().await?;
    java::install_jre(version, &state.locations.java_dir(), &state.fetch_semaphore).await
}
