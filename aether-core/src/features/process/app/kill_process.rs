use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{features::process::ProcessManager, shared::domain::AsyncUseCaseWithInputAndError};

pub struct KillProcessUseCase<PM: ProcessManager> {
    manager: Arc<PM>,
}

impl<PM: ProcessManager> KillProcessUseCase<PM> {
    pub fn new(manager: Arc<PM>) -> Self {
        Self { manager }
    }
}

#[async_trait]
impl<PM> AsyncUseCaseWithInputAndError for KillProcessUseCase<PM>
where
    PM: ProcessManager + Send + Sync,
{
    type Input = Uuid;
    type Output = ();
    type Error = crate::Error;

    async fn execute(&self, id: Self::Input) -> Result<Self::Output, Self::Error> {
        self.manager.kill(id).await
    }
}
