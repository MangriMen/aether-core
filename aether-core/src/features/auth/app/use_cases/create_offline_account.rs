use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::features::auth::{AccountType, AuthError, Credentials, CredentialsService, Username};

pub struct CreateOfflineAccountUseCase<CS: CredentialsService> {
    credentials_service: Arc<CS>,
}

use super::super::AccountOutput;

impl<CS: CredentialsService> CreateOfflineAccountUseCase<CS> {
    pub fn new(credentials_service: Arc<CS>) -> Self {
        Self {
            credentials_service,
        }
    }

    pub async fn execute(&self, username: String) -> Result<AccountOutput, AuthError> {
        let username = Username::parse(&username)?;

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
            .map(AccountOutput::from)
    }
}
