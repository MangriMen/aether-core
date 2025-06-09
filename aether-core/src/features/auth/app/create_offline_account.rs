use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::features::auth::{AccountDto, AccountType, AuthError, Credentials, CredentialsService};

pub struct CreateOfflineAccountUseCase<CS: CredentialsService> {
    credentials_service: Arc<CS>,
}

impl<CS: CredentialsService> CreateOfflineAccountUseCase<CS> {
    pub fn new(credentials_service: Arc<CS>) -> Self {
        Self {
            credentials_service,
        }
    }

    pub async fn execute(&self, username: String) -> Result<AccountDto, AuthError> {
        self.credentials_service
            .upsert(Credentials {
                id: Uuid::new_v4(),
                username,
                access_token: String::new(),
                refresh_token: String::new(),
                expires: Utc::now(),
                active: true,
                account_type: AccountType::Offline,
            })
            .await
            .map(AccountDto::from)
    }
}
