use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::features::auth::{AccountType, AuthError, Credentials, CredentialsStorage};

pub struct CreateOfflineAccountUseCase<CS: CredentialsStorage> {
    credentials_storage: Arc<CS>,
}

impl<CS: CredentialsStorage> CreateOfflineAccountUseCase<CS> {
    pub fn new(credentials_storage: Arc<CS>) -> Self {
        Self {
            credentials_storage,
        }
    }

    pub async fn execute(&self, username: String) -> Result<Uuid, AuthError> {
        self.credentials_storage
            .upsert(&Credentials {
                id: Uuid::new_v4(),
                username,
                access_token: String::new(),
                refresh_token: String::new(),
                expires: Utc::now(),
                active: true,
                account_type: AccountType::Offline,
            })
            .await
    }
}
