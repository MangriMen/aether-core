use std::path::Path;

use crate::{
    core::LauncherState,
    features::java::{self, infra::FsJavaStorage},
};

fn get_storage(state: &LauncherState) -> FsJavaStorage {
    FsJavaStorage::new(&state.locations.java_dir())
}

#[tracing::instrument]
pub async fn install(version: u32) -> crate::Result<java::Java> {
    let state = LauncherState::get().await?;
    java::install_java(&state, &get_storage(&state), version).await
}

#[tracing::instrument]
pub async fn get(version: u32) -> crate::Result<java::Java> {
    let state = LauncherState::get().await?;
    java::get_java(&get_storage(&state), version).await
}

#[tracing::instrument]
pub async fn get_from_path(path: &Path) -> crate::Result<java::Java> {
    java::get_java_from_path(path).await
}
