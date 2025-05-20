use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{features::process::ProcessStorage, shared::domain::AsyncUseCaseWithInputAndError};

pub struct KillProcessUseCase<PS: ProcessStorage> {
    process_storage: Arc<PS>,
}

impl<PS: ProcessStorage> KillProcessUseCase<PS> {
    pub fn new(process_storage: Arc<PS>) -> Self {
        Self { process_storage }
    }
}

#[async_trait]
impl<PS: ProcessStorage> AsyncUseCaseWithInputAndError for KillProcessUseCase<PS> {
    type Input = Uuid;
    type Output = ();
    type Error = crate::Error;

    async fn execute(&self, id: Self::Input) -> Result<Self::Output, Self::Error> {
        self.process_storage.kill(id).await
    }
}
