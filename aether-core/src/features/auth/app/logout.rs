use std::sync::Arc;

use uuid::Uuid;

use crate::features::auth::{AuthError, CredentialsService};

pub struct LogoutUseCase<CS: CredentialsService> {
    credentials_service: Arc<CS>,
}

impl<CS: CredentialsService> LogoutUseCase<CS> {
    pub fn new(credentials_service: Arc<CS>) -> Self {
        Self {
            credentials_service,
        }
    }

    pub async fn execute(&self, account_id: Uuid) -> Result<(), AuthError> {
        self.credentials_service.remove(account_id).await
    }
}
