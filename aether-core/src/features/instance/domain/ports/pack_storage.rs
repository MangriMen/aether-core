use async_trait::async_trait;

use crate::{
    features::instance::{Pack, PackFile},
    shared::StorageError,
};

#[async_trait]
pub trait PackStorage {
    async fn get_pack(&self, instance_id: &str) -> Result<Pack, StorageError>;

    async fn update_pack(&self, instance_id: &str, pack: &Pack) -> Result<(), StorageError>;

    async fn get_pack_file(
        &self,
        instance_id: &str,
        content_path: &str,
    ) -> Result<PackFile, StorageError>;

    async fn update_pack_file(
        &self,
        instance_id: &str,
        content_path: &str,
        pack_file: &PackFile,
    ) -> Result<(), StorageError>;

    async fn update_pack_file_many(
        &self,
        instance_id: &str,
        content_paths: &[String],
        pack_files: &[PackFile],
    ) -> Result<(), StorageError>;

    async fn remove_pack_file(
        &self,
        instance_id: &str,
        content_path: &str,
    ) -> Result<(), StorageError>;

    async fn remove_pack_file_many(
        &self,
        instance_id: &str,
        content_paths: &[String],
    ) -> Result<(), StorageError>;
}
