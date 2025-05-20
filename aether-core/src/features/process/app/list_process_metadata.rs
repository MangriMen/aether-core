use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::process::{MinecraftProcessMetadata, ProcessStorage},
    shared::domain::AsyncUseCase,
};

pub struct ListProcessMetadataUseCase<PS: ProcessStorage> {
    process_storage: Arc<PS>,
}

impl<PS: ProcessStorage> ListProcessMetadataUseCase<PS> {
    pub fn new(process_storage: Arc<PS>) -> Self {
        Self { process_storage }
    }
}

#[async_trait]
impl<PS: ProcessStorage> AsyncUseCase for ListProcessMetadataUseCase<PS> {
    type Output = Vec<MinecraftProcessMetadata>;

    async fn execute(&self) -> Self::Output {
        self.process_storage.list_metadata().await
    }
}
