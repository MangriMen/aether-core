use std::path::Path;

use crate::features::java::{
    infra::FsJavaInstallationService, Java, JavaError, JavaInstallationService,
};

pub async fn get_java_from_path(path: &Path) -> crate::Result<Java> {
    Ok(FsJavaInstallationService
        .locate_java(path)
        .await
        .ok_or_else(|| JavaError::InvalidPath {
            path: path.to_path_buf(),
        })?)
}
