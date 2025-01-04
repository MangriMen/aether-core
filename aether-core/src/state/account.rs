use uuid::Uuid;

use super::Credentials;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: Uuid,
    pub username: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AccountState {
    pub default: Option<Uuid>,
    pub accounts: Vec<Account>,
}

impl From<Credentials> for Account {
    fn from(credentials: Credentials) -> Self {
        Self {
            id: credentials.id,
            username: credentials.username,
        }
    }
}
