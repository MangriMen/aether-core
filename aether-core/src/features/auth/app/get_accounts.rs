use std::sync::Arc;

use crate::features::auth::{AccountDto, AuthError, CredentialsService};

pub struct GetAccountsUseCase<CS: CredentialsService> {
    credentials_service: Arc<CS>,
}

impl<CS: CredentialsService> GetAccountsUseCase<CS> {
    pub fn new(credentials_service: Arc<CS>) -> Self {
        Self {
            credentials_service,
        }
    }

    pub async fn execute(&self) -> Result<Vec<AccountDto>, AuthError> {
        let credentials = self.credentials_service.list().await?;
        Ok(credentials.into_iter().map(AccountDto::from).collect())
    }
}
