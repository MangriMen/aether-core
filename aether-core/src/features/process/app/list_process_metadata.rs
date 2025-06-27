use std::sync::Arc;

use crate::features::process::{MinecraftProcessMetadata, ProcessStorage};

pub struct ListProcessMetadataUseCase<PS: ProcessStorage> {
    process_storage: Arc<PS>,
}

impl<PS: ProcessStorage> ListProcessMetadataUseCase<PS> {
    pub fn new(process_storage: Arc<PS>) -> Self {
        Self { process_storage }
    }

    pub async fn execute(&self) -> Vec<MinecraftProcessMetadata> {
        self.process_storage.list_metadata().await
    }
}
