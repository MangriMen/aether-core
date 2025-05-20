use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Credentials;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: Uuid,
    pub active: bool,
    pub username: String,
    pub account_type: String,
}

impl From<Credentials> for Account {
    fn from(credentials: Credentials) -> Self {
        Self {
            id: credentials.id,
            username: credentials.username,
            active: credentials.active,
            // TODO: change when divide online and offline
            account_type: "offline".to_string(),
        }
    }
}
