use async_trait::async_trait;

use crate::features::instance::{InstanceError, Pack, PackFile};

#[async_trait]
pub trait PackStorage: Send + Sync {
    async fn get_pack(&self, instance_id: &str) -> Result<Pack, InstanceError>;

    async fn update_pack(&self, instance_id: &str, pack: &Pack) -> Result<(), InstanceError>;

    async fn get_pack_file(
        &self,
        instance_id: &str,
        content_path: &str,
    ) -> Result<PackFile, InstanceError>;

    async fn update_pack_file(
        &self,
        instance_id: &str,
        content_path: &str,
        pack_file: &PackFile,
    ) -> Result<(), InstanceError>;

    async fn update_pack_file_many(
        &self,
        instance_id: &str,
        content_paths: &[String],
        pack_files: &[PackFile],
    ) -> Result<(), InstanceError>;

    async fn remove_pack_file(
        &self,
        instance_id: &str,
        content_path: &str,
    ) -> Result<(), InstanceError>;

    async fn remove_pack_file_many(
        &self,
        instance_id: &str,
        content_paths: &[String],
    ) -> Result<(), InstanceError>;
}
