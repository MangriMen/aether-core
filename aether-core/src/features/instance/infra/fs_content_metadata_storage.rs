use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    features::{
        instance::{
            ContentMetadata, ContentMetadataEntry, ContentMetadataFile, ContentMetadataStorage,
        },
        settings::LocationInfo,
    },
    shared::{read_toml_async, remove_file, write_toml_async, StorageError},
};

pub struct FsContentMetadataStorage {
    location_info: Arc<LocationInfo>,
}

impl FsContentMetadataStorage {
    pub fn new(location_info: Arc<LocationInfo>) -> Self {
        Self { location_info }
    }

    fn get_content_metadata_path(&self, instance_id: &str) -> PathBuf {
        self.location_info.instance_content_metadata(instance_id)
    }

    fn get_content_metadata_file_path(&self, instance_id: &str, content_path: &str) -> PathBuf {
        self.location_info
            .instance_pack_dir(instance_id)
            .join(content_path)
            .with_extension("toml")
    }

    async fn read<T>(&self, file: &Path) -> Result<T, StorageError>
    where
        T: DeserializeOwned,
    {
        if !file.exists() {
            return Err(StorageError::NotFound {
                path: file.to_path_buf(),
            });
        }

        read_toml_async(&file)
            .await
            .map_err(|err| StorageError::ReadError(err.raw.to_string()))
    }

    async fn ensure_read<T>(&self, file: &Path) -> Result<T, StorageError>
    where
        T: Serialize + DeserializeOwned + Default,
    {
        if !file.exists() {
            let default = T::default();
            self.write(file, &default).await?;
            return Ok(default);
        }

        read_toml_async(&file)
            .await
            .map_err(|err| StorageError::ReadError(err.raw.to_string()))
    }

    async fn write<T>(&self, file: &Path, data: &T) -> Result<(), StorageError>
    where
        T: Serialize,
    {
        write_toml_async(&file, &data)
            .await
            .map_err(|err| StorageError::WriteError(err.raw.to_string()))
    }
}

#[async_trait]
impl ContentMetadataStorage for FsContentMetadataStorage {
    async fn get_content_metadata(
        &self,
        instance_id: &str,
    ) -> Result<ContentMetadata, StorageError> {
        self.ensure_read(&self.get_content_metadata_path(instance_id))
            .await
    }

    async fn update_content_metadata(
        &self,
        instance_id: &str,
        content_metadata: &ContentMetadata,
    ) -> Result<(), StorageError> {
        self.write(
            &self.get_content_metadata_path(instance_id),
            &content_metadata,
        )
        .await
    }

    async fn get_content_metadata_file(
        &self,
        instance_id: &str,
        content_path: &str,
    ) -> Result<ContentMetadataFile, StorageError> {
        self.read(&self.get_content_metadata_file_path(instance_id, content_path))
            .await
    }

    async fn update_content_metadata_file(
        &self,
        instance_id: &str,
        content_path: &str,
        content_metadata_file: &ContentMetadataFile,
    ) -> Result<(), StorageError> {
        self.update_content_metadata_file_many(
            instance_id,
            &[content_path.to_string()],
            &[content_metadata_file.clone()],
        )
        .await
    }

    async fn update_content_metadata_file_many(
        &self,
        instance_id: &str,
        content_paths: &[String],
        content_metadata_file: &[ContentMetadataFile],
    ) -> Result<(), StorageError> {
        let mut new_content_metadata_file_paths = Vec::new();

        for (content_path, content_metadata_file) in content_paths.iter().zip(content_metadata_file)
        {
            let content_metadata_file_path =
                self.get_content_metadata_file_path(instance_id, content_path);

            if !content_metadata_file_path.exists() {
                new_content_metadata_file_paths.push(content_path);
            }

            self.write(&content_metadata_file_path, &content_metadata_file)
                .await?;
        }

        if !new_content_metadata_file_paths.is_empty() {
            let mut content_metadata = self.get_content_metadata(instance_id).await?;

            let content_metadata_entries: Vec<ContentMetadataEntry> =
                new_content_metadata_file_paths
                    .iter()
                    .map(|path| ContentMetadataEntry {
                        file: path.to_string(),
                    })
                    .collect();

            content_metadata
                .files
                .extend_from_slice(&content_metadata_entries);

            self.update_content_metadata(instance_id, &content_metadata)
                .await?;
        }

        Ok(())
    }

    async fn remove_content_metadata_file(
        &self,
        instance_id: &str,
        content_path: &str,
    ) -> Result<(), StorageError> {
        self.remove_content_metadata_file_many(instance_id, &[content_path.to_string()])
            .await
    }

    async fn remove_content_metadata_file_many(
        &self,
        instance_id: &str,
        content_paths: &[String],
    ) -> Result<(), StorageError> {
        let mut success_deleted_content_paths = Vec::new();

        for content_path in content_paths {
            let content_metadata_file_path =
                self.get_content_metadata_file_path(instance_id, content_path);

            if remove_file(&content_metadata_file_path)
                .await
                .map_err(|err| StorageError::WriteError(err.to_string()))
                .is_ok()
            {
                success_deleted_content_paths.push(content_path);
            }
        }

        let mut content_metadata = self.get_content_metadata(instance_id).await?;
        content_metadata
            .files
            .retain(|entry| success_deleted_content_paths.contains(&&entry.file));
        self.update_content_metadata(instance_id, &content_metadata)
            .await
    }
}
