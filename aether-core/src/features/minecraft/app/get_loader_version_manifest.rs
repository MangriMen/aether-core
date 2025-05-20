use std::sync::Arc;

use async_trait::async_trait;
use daedalus::modded;

use crate::{
    features::minecraft::ReadMetadataStorage, shared::domain::AsyncUseCaseWithInputAndError,
};

pub struct GetLoaderVersionManifestUseCase<MS: ReadMetadataStorage> {
    metadata_storage: Arc<MS>,
}

impl<MS: ReadMetadataStorage> GetLoaderVersionManifestUseCase<MS> {
    pub fn new(metadata_storage: Arc<MS>) -> Self {
        Self { metadata_storage }
    }
}

#[async_trait]
impl<MS: ReadMetadataStorage> AsyncUseCaseWithInputAndError
    for GetLoaderVersionManifestUseCase<MS>
{
    type Input = String;
    type Output = modded::Manifest;
    type Error = crate::Error;

    async fn execute(&self, loader: Self::Input) -> Result<Self::Output, Self::Error> {
        Ok(self
            .metadata_storage
            .get_loader_version_manifest(&loader)
            .await?
            .value)
    }
}
