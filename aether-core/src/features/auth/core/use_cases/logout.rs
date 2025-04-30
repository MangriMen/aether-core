use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{features::auth::CredentialsStorage, shared::domain::AsyncUseCase};

pub struct LogoutUseCase<CS: CredentialsStorage> {
    storage: Arc<CS>,
}

impl<CS: CredentialsStorage> LogoutUseCase<CS> {
    pub fn new(storage: Arc<CS>) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl<CS> AsyncUseCase for LogoutUseCase<CS>
where
    CS: CredentialsStorage + Send + Sync,
{
    type Input = Uuid;
    type Output = ();
    type Error = crate::Error;

    async fn execute(&self, uuid: Self::Input) -> Result<Self::Output, Self::Error> {
        self.storage.remove(&uuid).await
    }
}
