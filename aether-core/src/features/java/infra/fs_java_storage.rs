use crate::{
    features::java::{Java, JavaStorage},
    shared::infra::{AsyncFileDb, AsyncJsonDb},
};
use async_trait::async_trait;
use std::path::Path;

pub struct FsJavaStorage {
    db: AsyncJsonDb<Vec<Java>>,
}

impl FsJavaStorage {
    pub fn new(java_dir: &Path) -> Self {
        Self {
            db: AsyncJsonDb::new(java_dir.to_path_buf().join("java_versions.json")),
        }
    }

    #[inline]
    fn get_default() -> Vec<Java> {
        Vec::default()
    }
}

#[async_trait]
impl JavaStorage for FsJavaStorage {
    async fn get(&self, version: u32) -> crate::Result<Option<Java>> {
        Ok(self
            .db
            .ensure_read(Self::get_default)
            .await?
            .iter()
            .find(|x| x.major_version == version)
            .cloned())
    }

    async fn upsert(&self, java: &Java) -> crate::Result<()> {
        let mut java_versions = self.db.ensure_read(Self::get_default).await?;

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

        self.db.write(&java_versions).await
    }
}
