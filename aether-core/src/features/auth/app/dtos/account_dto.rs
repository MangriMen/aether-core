use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::features::auth::{AccountType, Credentials};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AccountDto {
    pub id: Uuid,
    pub username: String,
    pub active: bool,
    pub account_type: AccountType,
}

impl From<Credentials> for AccountDto {
    fn from(credentials: Credentials) -> Self {
        Self {
            id: credentials.id,
            username: credentials.username,
            active: credentials.active,
            account_type: credentials.account_type,
        }
    }
}
