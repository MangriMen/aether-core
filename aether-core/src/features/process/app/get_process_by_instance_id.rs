use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::process::{MinecraftProcessMetadata, ProcessManager},
    shared::domain::AsyncUseCaseWithInput,
};

pub struct GetProcessByInstanceIdUseCase<PM: ProcessManager> {
    manager: Arc<PM>,
}

impl<PM: ProcessManager> GetProcessByInstanceIdUseCase<PM> {
    pub fn new(manager: Arc<PM>) -> Self {
        Self { manager }
    }
}

#[async_trait]
impl<PM> AsyncUseCaseWithInput for GetProcessByInstanceIdUseCase<PM>
where
    PM: ProcessManager + Send + Sync,
{
    type Input = String;
    type Output = Vec<MinecraftProcessMetadata>;

    async fn execute(&self, id: Self::Input) -> Self::Output {
        self.manager
            .list()
            .iter()
            .filter(|x| x.id == id)
            .cloned()
            .collect()
    }
}
