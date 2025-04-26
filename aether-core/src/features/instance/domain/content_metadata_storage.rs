use async_trait::async_trait;

use crate::shared::StorageError;

use super::{ContentMetadata, ContentMetadataFile};

#[async_trait]
pub trait ContentMetadataStorage {
    async fn get_content_metadata(
        &self,
        instance_id: &str,
    ) -> Result<ContentMetadata, StorageError>;

    async fn update_content_metadata(
        &self,
        instance_id: &str,
        content_metadata: &ContentMetadata,
    ) -> Result<(), StorageError>;

    async fn get_content_metadata_file(
        &self,
        instance_id: &str,
        content_path: &str,
    ) -> Result<ContentMetadataFile, StorageError>;

    async fn update_content_metadata_file(
        &self,
        instance_id: &str,
        content_path: &str,
        content_metadata_file: &ContentMetadataFile,
    ) -> Result<(), StorageError>;

    async fn update_content_metadata_file_many(
        &self,
        instance_id: &str,
        content_paths: &[String],
        content_metadata_files: &[ContentMetadataFile],
    ) -> Result<(), StorageError>;

    async fn remove_content_metadata_file(
        &self,
        instance_id: &str,
        content_path: &str,
    ) -> Result<(), StorageError>;

    async fn remove_content_metadata_file_many(
        &self,
        instance_id: &str,
        content_paths: &[String],
    ) -> Result<(), StorageError>;
}
