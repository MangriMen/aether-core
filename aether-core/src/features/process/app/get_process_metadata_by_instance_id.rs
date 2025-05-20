use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::process::{MinecraftProcessMetadata, ProcessStorage},
    shared::domain::AsyncUseCaseWithInput,
};

pub struct GetProcessMetadataByInstanceIdUseCase<PS: ProcessStorage> {
    process_storage: Arc<PS>,
}

impl<PS: ProcessStorage> GetProcessMetadataByInstanceIdUseCase<PS> {
    pub fn new(process_storage: Arc<PS>) -> Self {
        Self { process_storage }
    }
}

#[async_trait]
impl<PS: ProcessStorage> AsyncUseCaseWithInput for GetProcessMetadataByInstanceIdUseCase<PS> {
    type Input = String;
    type Output = Vec<MinecraftProcessMetadata>;

    async fn execute(&self, id: Self::Input) -> Self::Output {
        self.process_storage
            .list_metadata()
            .await
            .iter()
            .filter(|x| x.instance_id == id)
            .cloned()
            .collect()
    }
}
