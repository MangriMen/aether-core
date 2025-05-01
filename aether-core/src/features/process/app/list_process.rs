use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::process::{MinecraftProcessMetadata, ProcessManager},
    shared::domain::AsyncUseCase,
};

pub struct ListProcessUseCase<PM: ProcessManager> {
    manager: Arc<PM>,
}

impl<PM: ProcessManager> ListProcessUseCase<PM> {
    pub fn new(manager: Arc<PM>) -> Self {
        Self { manager }
    }
}

#[async_trait]
impl<PM> AsyncUseCase for ListProcessUseCase<PM>
where
    PM: ProcessManager + Send + Sync,
{
    type Output = Vec<MinecraftProcessMetadata>;

    async fn execute(&self) -> Self::Output {
        self.manager.list()
    }
}
