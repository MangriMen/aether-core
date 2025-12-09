use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;

use crate::{
    features::{
        instance::{InstanceError, Pack, PackEntry, PackFile, PackStorage},
        settings::LocationInfo,
    },
    shared::{ensure_read_toml_async, read_toml_async, remove_file, write_toml_async},
};

pub struct FsPackStorage {
    location_info: Arc<LocationInfo>,
}

impl FsPackStorage {
    pub fn new(location_info: Arc<LocationInfo>) -> Self {
        Self { location_info }
    }

    fn get_pack_path(&self, instance_id: &str) -> PathBuf {
        self.location_info.instance_pack(instance_id)
    }

    fn get_pack_file_path(&self, instance_id: &str, content_path: &str) -> PathBuf {
        self.location_info
            .instance_pack_dir(instance_id)
            .join(content_path)
            .with_extension("toml")
    }
}

#[async_trait]
impl PackStorage for FsPackStorage {
    async fn get_pack(&self, instance_id: &str) -> Result<Pack, InstanceError> {
        Ok(ensure_read_toml_async(&self.get_pack_path(instance_id)).await?)
    }

    async fn update_pack(&self, instance_id: &str, pack: &Pack) -> Result<(), InstanceError> {
        write_toml_async(&self.get_pack_path(instance_id), &pack).await?;
        Ok(())
    }

    async fn get_pack_file(
        &self,
        instance_id: &str,
        content_path: &str,
    ) -> Result<PackFile, InstanceError> {
        Ok(read_toml_async(&self.get_pack_file_path(instance_id, content_path)).await?)
    }

    async fn update_pack_file(
        &self,
        instance_id: &str,
        content_path: &str,
        pack_file: &PackFile,
    ) -> Result<(), InstanceError> {
        self.update_pack_file_many(
            instance_id,
            &[content_path.to_string()],
            std::slice::from_ref(pack_file),
        )
        .await
    }

    async fn update_pack_file_many(
        &self,
        instance_id: &str,
        content_paths: &[String],
        pack_file: &[PackFile],
    ) -> Result<(), InstanceError> {
        let mut new_pack_file_paths = Vec::new();

        for (content_path, pack_file) in content_paths.iter().zip(pack_file) {
            let pack_file_path = self.get_pack_file_path(instance_id, content_path);

            if !pack_file_path.exists() {
                new_pack_file_paths.push(content_path);
            }

            write_toml_async(&pack_file_path, &pack_file).await?;
        }

        if !new_pack_file_paths.is_empty() {
            let mut pack = self.get_pack(instance_id).await?;

            let pack_entries: Vec<PackEntry> = new_pack_file_paths
                .iter()
                .map(|path| PackEntry {
                    file: path.to_string(),
                })
                .collect();

            pack.files.extend_from_slice(&pack_entries);

            self.update_pack(instance_id, &pack).await?;
        }

        Ok(())
    }

    async fn remove_pack_file(
        &self,
        instance_id: &str,
        content_path: &str,
    ) -> Result<(), InstanceError> {
        self.remove_pack_file_many(instance_id, &[content_path.to_string()])
            .await
    }

    async fn remove_pack_file_many(
        &self,
        instance_id: &str,
        content_paths: &[String],
    ) -> Result<(), InstanceError> {
        let mut success_deleted_content_paths = Vec::new();

        for content_path in content_paths {
            let pack_file_path = self.get_pack_file_path(instance_id, content_path);

            if remove_file(&pack_file_path).await.is_ok() {
                success_deleted_content_paths.push(content_path);
            }
        }

        let mut pack = self.get_pack(instance_id).await?;
        pack.files
            .retain(|entry| !success_deleted_content_paths.contains(&&entry.file));
        pack.files.dedup_by_key(|item| item.file.to_string());
        self.update_pack(instance_id, &pack).await
    }
}
