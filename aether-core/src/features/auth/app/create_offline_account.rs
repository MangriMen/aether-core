use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use crate::{
    features::auth::{Credentials, CredentialsStorage},
    shared::domain::AsyncUseCaseWithInputAndError,
};

pub struct CreateOfflineAccountUseCase<CS: CredentialsStorage> {
    storage: Arc<CS>,
}

impl<CS: CredentialsStorage> CreateOfflineAccountUseCase<CS> {
    pub fn new(storage: Arc<CS>) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl<CS> AsyncUseCaseWithInputAndError for CreateOfflineAccountUseCase<CS>
where
    CS: CredentialsStorage + Send + Sync,
{
    type Input = String;
    type Output = Uuid;
    type Error = crate::Error;

    async fn execute(&self, username: Self::Input) -> Result<Self::Output, Self::Error> {
        self.storage
            .upsert(&Credentials {
                id: Uuid::new_v4(),
                username,
                access_token: String::new(),
                refresh_token: String::new(),
                expires: Utc::now(),
                active: true,
            })
            .await
    }
}
