use std::sync::Arc;

use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::features::auth::{
    AccountOutput, AccountType, AuthError, Credentials, CredentialsService, Username,
};

pub struct CreateOfflineAccountUseCase<CS: CredentialsService> {
    credentials_service: Arc<CS>,
}

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
                access_token: "null".to_string(),
                refresh_token: "null".to_string(),
                expires: Utc::now() + Duration::days(365 * 99),
                active: true,
                account_type: AccountType::Offline,
            })
            .await
            .map(AccountOutput::from)
    }
}
