use std::path::PathBuf;

use crate::{
    state::{Java, LauncherState},
    utils::jre,
};

// Validates JRE at a given path
#[tracing::instrument]
pub async fn check_jre(path: PathBuf) -> crate::Result<Option<Java>> {
    Ok(jre::check_java_at_filepath(&path).await)
}

#[tracing::instrument]
pub async fn get_or_download_java(version: u32) -> crate::Result<Java> {
    let state = LauncherState::get().await?;

    let (java_path, add_java) = if let Some(java) = Java::get(&state, version).await? {
        (PathBuf::from(java.path), false)
    } else {
        (crate::jre::auto_install_java(version).await?, true)
    };

    let java = crate::api::jre::check_jre(java_path.clone())
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::LauncherError(format!(
                "Java path invalid or non-functional: {:?}",
                java_path
            ))
        })?;

    if add_java {
        java.update(&state).await?;
    }

    Ok(java)
}
