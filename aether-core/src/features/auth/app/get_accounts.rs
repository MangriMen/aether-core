use std::sync::Arc;

use crate::features::auth::{Account, CredentialsStorage};

pub struct GetAccountsUseCase<CS: CredentialsStorage> {
    credentials_storage: Arc<CS>,
}

impl<CS: CredentialsStorage> GetAccountsUseCase<CS> {
    pub fn new(credentials_storage: Arc<CS>) -> Self {
        Self {
            credentials_storage,
        }
    }

    pub async fn execute(&self) -> crate::Result<Vec<Account>> {
        let credentials = self.credentials_storage.list().await?;
        Ok(credentials.into_iter().map(Account::from).collect())
    }
}
