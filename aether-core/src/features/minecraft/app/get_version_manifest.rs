use std::sync::Arc;

use async_trait::async_trait;
use daedalus::minecraft;

use crate::{features::minecraft::ReadMetadataStorage, shared::domain::AsyncUseCaseWithError};

pub struct GetVersionManifestUseCase<MS: ReadMetadataStorage> {
    storage: Arc<MS>,
}

impl<MS: ReadMetadataStorage> GetVersionManifestUseCase<MS> {
    pub fn new(storage: Arc<MS>) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl<MS> AsyncUseCaseWithError for GetVersionManifestUseCase<MS>
where
    MS: ReadMetadataStorage + Send + Sync,
{
    type Output = minecraft::VersionManifest;
    type Error = crate::Error;

    async fn execute(&self) -> Result<Self::Output, Self::Error> {
        Ok(self.storage.get_version_manifest().await?.value)
    }
}
