use std::sync::Arc;

use async_trait::async_trait;
use daedalus::minecraft;

use crate::{features::minecraft::ReadMetadataStorage, shared::domain::AsyncUseCaseWithError};

pub struct GetVersionManifestUseCase<MS: ReadMetadataStorage> {
    metadata_storage: Arc<MS>,
}

impl<MS: ReadMetadataStorage> GetVersionManifestUseCase<MS> {
    pub fn new(metadata_storage: Arc<MS>) -> Self {
        Self { metadata_storage }
    }
}

#[async_trait]
impl<MS: ReadMetadataStorage> AsyncUseCaseWithError for GetVersionManifestUseCase<MS> {
    type Output = minecraft::VersionManifest;
    type Error = crate::Error;

    async fn execute(&self) -> Result<Self::Output, Self::Error> {
        Ok(self.metadata_storage.get_version_manifest().await?.value)
    }
}
