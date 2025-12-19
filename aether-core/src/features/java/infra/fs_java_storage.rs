use crate::{
    features::java::{Java, JavaStorage, JavaStorageError},
    shared::{read_json_async, write_json_async},
};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

pub struct FsJavaStorage {
    java_versions_file: PathBuf,
}

impl FsJavaStorage {
    pub fn new(java_dir: &Path) -> Self {
        Self {
            java_versions_file: java_dir.join("java_versions.json"),
        }
    }

    async fn ensure_read(&self) -> Result<Vec<Java>, JavaStorageError> {
        if !self.java_versions_file.exists() {
            let default = Vec::<Java>::default();
            self.write(&default).await?;
            return Ok(default);
        }

        Ok(read_json_async(&self.java_versions_file).await?)
    }

    async fn write(&self, data: &Vec<Java>) -> Result<(), JavaStorageError> {
        Ok(write_json_async(&self.java_versions_file, &data).await?)
    }
}

#[async_trait]
impl JavaStorage for FsJavaStorage {
    async fn list(&self) -> Result<Vec<Java>, JavaStorageError> {
        self.ensure_read().await
    }

    async fn get(&self, version: u32) -> Result<Option<Java>, JavaStorageError> {
        Ok(self
            .ensure_read()
            .await?
            .iter()
            .find(|x| x.major_version == version)
            .cloned())
    }

    async fn upsert(&self, java: &Java) -> Result<(), JavaStorageError> {
        let mut java_versions = self.ensure_read().await?;

        match java_versions
            .iter_mut()
            .find(|x| x.major_version == java.major_version)
        {
            Some(existing) => {
                log::debug!("Updating existing Java version {}", java.major_version);
                existing.clone_from(java);
            }
            None => {
                log::debug!("Inserting new Java version {}", java.major_version);
                java_versions.push(java.clone());
            }
        }

        self.write(&java_versions).await
    }
}
