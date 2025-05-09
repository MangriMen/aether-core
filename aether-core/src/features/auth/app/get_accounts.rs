use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::auth::{Account, CredentialsStorage},
    shared::domain::AsyncUseCaseWithError,
};

pub struct GetAccountsUseCase<CS: CredentialsStorage> {
    credentials_storage: Arc<CS>,
}

impl<CS: CredentialsStorage> GetAccountsUseCase<CS> {
    pub fn new(credentials_storage: Arc<CS>) -> Self {
        Self {
            credentials_storage,
        }
    }
}

#[async_trait]
impl<CS> AsyncUseCaseWithError for GetAccountsUseCase<CS>
where
    CS: CredentialsStorage + Send + Sync,
{
    type Output = Vec<Account>;
    type Error = crate::Error;

    async fn execute(&self) -> Result<Self::Output, Self::Error> {
        let credentials = self.credentials_storage.list().await?;
        Ok(credentials.into_iter().map(Account::from).collect())
    }
}
