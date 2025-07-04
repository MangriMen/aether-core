use std::sync::Arc;

use uuid::Uuid;

use crate::features::auth::{AccountOutput, AuthError, CredentialsService};

pub struct SetActiveAccountUseCase<CS: CredentialsService> {
    credentials_service: Arc<CS>,
}

impl<CS: CredentialsService> SetActiveAccountUseCase<CS> {
    pub fn new(credentials_service: Arc<CS>) -> Self {
        Self {
            credentials_service,
        }
    }

    pub async fn execute(&self, account_id: Uuid) -> Result<AccountOutput, AuthError> {
        self.credentials_service
            .set_active(account_id)
            .await
            .map(AccountOutput::from)
    }
}
