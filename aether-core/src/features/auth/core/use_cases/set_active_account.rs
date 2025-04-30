use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{features::auth::CredentialsStorage, shared::domain::AsyncUseCase};

pub struct SetActiveAccountUseCase<CS: CredentialsStorage> {
    storage: Arc<CS>,
}

impl<CS: CredentialsStorage> SetActiveAccountUseCase<CS> {
    pub fn new(storage: Arc<CS>) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl<CS> AsyncUseCase for SetActiveAccountUseCase<CS>
where
    CS: CredentialsStorage + Send + Sync,
{
    type Input = Uuid;
    type Output = ();
    type Error = crate::Error;

    async fn execute(&self, id: Self::Input) -> Result<Self::Output, Self::Error> {
        self.storage.set_active(&id).await
    }
}
