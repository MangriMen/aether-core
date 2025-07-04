use std::sync::Arc;

use crate::features::process::{MinecraftProcessMetadata, ProcessStorage};

pub struct GetProcessMetadataByInstanceIdUseCase<PS: ProcessStorage> {
    process_storage: Arc<PS>,
}

impl<PS: ProcessStorage> GetProcessMetadataByInstanceIdUseCase<PS> {
    pub fn new(process_storage: Arc<PS>) -> Self {
        Self { process_storage }
    }

    pub async fn execute(&self, instance_id: String) -> Vec<MinecraftProcessMetadata> {
        self.process_storage
            .list_metadata()
            .await
            .iter()
            .filter(|x| x.instance_id == instance_id)
            .cloned()
            .collect()
    }
}
