use std::sync::Arc;

use uuid::Uuid;

use crate::features::auth::{AuthError, CredentialsStorage};

pub struct SetActiveAccountUseCase<CS: CredentialsStorage> {
    credentials_storage: Arc<CS>,
}

impl<CS: CredentialsStorage> SetActiveAccountUseCase<CS> {
    pub fn new(credentials_storage: Arc<CS>) -> Self {
        Self {
            credentials_storage,
        }
    }

    pub async fn execute(&self, account_id: Uuid) -> Result<(), AuthError> {
        self.credentials_storage.set_active(account_id).await
    }
}
