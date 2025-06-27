use std::sync::Arc;

use crate::features::auth::{AccountOutput, AuthError, CredentialsService};

pub struct GetAccountsUseCase<CS: CredentialsService> {
    credentials_service: Arc<CS>,
}

impl<CS: CredentialsService> GetAccountsUseCase<CS> {
    pub fn new(credentials_service: Arc<CS>) -> Self {
        Self {
            credentials_service,
        }
    }

    pub async fn execute(&self) -> Result<Vec<AccountOutput>, AuthError> {
        let credentials = self.credentials_service.list().await?;
        Ok(credentials.iter().map(AccountOutput::from).collect())
    }
}
