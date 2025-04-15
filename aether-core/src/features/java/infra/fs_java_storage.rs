use crate::{
    features::java::{Java, JavaStorage},
    shared::infra::AsyncFsDb,
};
use async_trait::async_trait;
use std::path::Path;

pub struct FsJavaStorage {
    db: AsyncFsDb<Vec<Java>>,
}

impl FsJavaStorage {
    pub fn new(java_dir: &Path) -> Self {
        Self {
            db: AsyncFsDb::new(java_dir.to_path_buf()),
        }
    }
}

#[async_trait]
impl JavaStorage for FsJavaStorage {
    async fn get(&self, version: u32) -> crate::Result<Option<Java>> {
        Ok(self
            .db
            .read_file_contents()
            .await?
            .iter()
            .find(|x| x.major_version == version)
            .cloned())
    }

    async fn upsert(&self, java: &Java) -> crate::Result<()> {
        let mut java_versions = self.db.read_file_contents().await?;

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

        self.db.write_file_contents(java_versions).await
    }
}
