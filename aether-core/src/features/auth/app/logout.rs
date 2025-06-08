use std::sync::Arc;

use uuid::Uuid;

use crate::features::auth::{AuthError, CredentialsStorage};

pub struct LogoutUseCase<CS: CredentialsStorage> {
    credentials_storage: Arc<CS>,
}

impl<CS: CredentialsStorage> LogoutUseCase<CS> {
    pub fn new(credentials_storage: Arc<CS>) -> Self {
        Self {
            credentials_storage,
        }
    }

    pub async fn execute(&self, account_id: Uuid) -> Result<(), AuthError> {
        self.credentials_storage.remove(account_id).await
    }
}
