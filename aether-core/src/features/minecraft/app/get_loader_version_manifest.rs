use std::sync::Arc;

use async_trait::async_trait;
use daedalus::modded;

use crate::{
    features::minecraft::ReadMetadataStorage, shared::domain::AsyncUseCaseWithInputAndError,
};

pub struct GetLoaderVersionManifestUseCase<MS: ReadMetadataStorage> {
    storage: Arc<MS>,
}

impl<MS: ReadMetadataStorage> GetLoaderVersionManifestUseCase<MS> {
    pub fn new(storage: Arc<MS>) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl<MS> AsyncUseCaseWithInputAndError for GetLoaderVersionManifestUseCase<MS>
where
    MS: ReadMetadataStorage + Send + Sync,
{
    type Input = String;
    type Output = modded::Manifest;
    type Error = crate::Error;

    async fn execute(&self, loader: Self::Input) -> Result<Self::Output, Self::Error> {
        Ok(self
            .storage
            .get_loader_version_manifest(&loader)
            .await?
            .value)
    }
}
