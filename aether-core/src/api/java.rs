use std::path::Path;

use crate::state::{Java, LauncherState};

// Install JRE
#[tracing::instrument]
pub async fn install(version: u32) -> crate::Result<Java> {
    let state = LauncherState::get().await?;

    let path = crate::features::java::install_jre(version).await?;
    let java = Java::from_path(&path).await;

    if let Ok(java) = &java {
        java.upsert(&state).await?;
    }

    java
}

// Get JRE if installed
#[tracing::instrument]
pub async fn get(version: u32) -> crate::Result<Java> {
    let state = LauncherState::get().await?;
    let java = Java::get(&state, version).await?;

    if let Some(java) = java {
        Java::from_path(Path::new(&java.path)).await
    } else {
        Err(crate::ErrorKind::LauncherError(format!("Java {} not found", version)).as_error())
    }
}

#[tracing::instrument]
pub async fn get_from_path(path: &Path) -> crate::Result<Java> {
    Java::from_path(path).await
}
