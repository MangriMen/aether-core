use std::path::Path;

use crate::{
    core::LauncherState,
    features::java::{infra::FsJavaStorage, Java, JavaStorage},
};

// Install JRE
#[tracing::instrument]
pub async fn install(version: u32) -> crate::Result<Java> {
    let state = LauncherState::get().await?;

    let path = crate::features::java::install_jre(version).await?;
    let java_storage = FsJavaStorage;
    let java = java_storage.create_from_path(&path).await;

    if let Ok(java) = &java {
        java_storage.upsert(&state, java).await?;
    }

    java
}

// Get JRE if installed
#[tracing::instrument]
pub async fn get(version: u32) -> crate::Result<Java> {
    let state = LauncherState::get().await?;

    let java_storage = FsJavaStorage;
    let java = java_storage.get(&state, version).await?;

    if let Some(java) = java {
        java_storage.create_from_path(Path::new(&java.path)).await
    } else {
        Err(crate::ErrorKind::LauncherError(format!("Java {} not found", version)).as_error())
    }
}

#[tracing::instrument]
pub async fn get_from_path(path: &Path) -> crate::Result<Java> {
    FsJavaStorage.create_from_path(path).await
}
