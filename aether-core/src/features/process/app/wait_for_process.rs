use std::sync::Arc;

use uuid::Uuid;

use crate::features::process::ProcessStorage;

pub struct WaitForProcessUseCase<PS: ProcessStorage> {
    process_storage: Arc<PS>,
}

impl<PS: ProcessStorage> WaitForProcessUseCase<PS> {
    pub fn new(process_storage: Arc<PS>) -> Self {
        Self { process_storage }
    }

    pub async fn execute(&self, id: Uuid) -> crate::Result<()> {
        self.process_storage.wait_for(id).await
    }
}
