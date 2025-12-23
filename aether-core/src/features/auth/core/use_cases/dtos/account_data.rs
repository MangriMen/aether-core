use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::features::auth::{AccountType, Credentials, Username};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AccountData {
    pub id: Uuid,
    pub username: Username,
    pub active: bool,
    pub account_type: AccountType,
}

impl From<&Credentials> for AccountData {
    fn from(credentials: &Credentials) -> Self {
        Self {
            id: credentials.id,
            username: credentials.username.to_owned(),
            active: credentials.active,
            account_type: credentials.account_type,
        }
    }
}

impl From<Credentials> for AccountData {
    fn from(credentials: Credentials) -> Self {
        AccountData::from(&credentials)
    }
}
