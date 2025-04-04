use crate::{
    features::java::{application::storage::JavaStorage, construct_java_from_jre, domain::Java},
    state::LauncherState,
    utils::io::{read_json_async, write_json_async},
};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

pub struct FsJavaStorage;

impl FsJavaStorage {
    fn get_versions_file_path(state: &LauncherState) -> PathBuf {
        state.locations.java_dir().join("versions.json")
    }

    async fn ensure_versions_file_exists(path: &Path) -> crate::Result<()> {
        if !path.exists() {
            log::info!(
                "Java versions file not found, creating new one at {}",
                path.display()
            );
            write_json_async(path, Vec::<Java>::new()).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl JavaStorage for FsJavaStorage {
    async fn create_from_path(&self, path: &Path) -> crate::Result<Java> {
        construct_java_from_jre(path).await.ok_or_else(|| {
            crate::ErrorKind::LauncherError(format!("Invalid Java path: {:?}", path)).as_error()
        })
    }

    async fn get(&self, state: &LauncherState, version: u32) -> crate::Result<Option<Java>> {
        let path = Self::get_versions_file_path(state);
        Self::ensure_versions_file_exists(&path).await?;

        let java_versions = read_json_async::<Vec<Java>>(path).await?;

        let found = java_versions.iter().find(|x| x.major_version == version);
        if let Some(java) = found {
            log::info!("Found Java version: {}", java.major_version);
            Ok(Some(java.clone()))
        } else {
            Ok(None)
        }
    }

    async fn upsert(&self, state: &LauncherState, java: &Java) -> crate::Result<()> {
        let path = Self::get_versions_file_path(state);

        let mut java_versions = read_json_async::<Vec<Java>>(&path).await?;

        match java_versions
            .iter_mut()
            .find(|x| x.major_version == java.major_version)
        {
            Some(existing) => {
                log::info!("Updating existing Java version {}", java.major_version);
                existing.clone_from(java);
            }
            None => {
                log::info!("Inserting new Java version {}", java.major_version);
                java_versions.push(java.clone());
            }
        }

        write_json_async(&path, &java_versions).await
    }
}
