use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{features::auth::CredentialsStorage, shared::domain::AsyncUseCaseWithInputAndError};

pub struct LogoutUseCase<CS: CredentialsStorage> {
    credentials_storage: Arc<CS>,
}

impl<CS: CredentialsStorage> LogoutUseCase<CS> {
    pub fn new(credentials_storage: Arc<CS>) -> Self {
        Self {
            credentials_storage,
        }
    }
}

#[async_trait]
impl<CS> AsyncUseCaseWithInputAndError for LogoutUseCase<CS>
where
    CS: CredentialsStorage + Send + Sync,
{
    type Input = Uuid;
    type Output = ();
    type Error = crate::Error;

    async fn execute(&self, uuid: Self::Input) -> Result<Self::Output, Self::Error> {
        self.credentials_storage.remove(&uuid).await
    }
}
