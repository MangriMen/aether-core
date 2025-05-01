use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{features::process::ProcessManager, shared::domain::AsyncUseCaseWithInputAndError};

pub struct WaitForProcessUseCase<PM: ProcessManager> {
    manager: Arc<PM>,
}

impl<PM: ProcessManager> WaitForProcessUseCase<PM> {
    pub fn new(manager: Arc<PM>) -> Self {
        Self { manager }
    }
}

#[async_trait]
impl<PM> AsyncUseCaseWithInputAndError for WaitForProcessUseCase<PM>
where
    PM: ProcessManager + Send + Sync,
{
    type Input = Uuid;
    type Output = ();
    type Error = crate::Error;

    async fn execute(&self, id: Self::Input) -> Result<Self::Output, Self::Error> {
        self.manager.wait_for(id).await
    }
}
