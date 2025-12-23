use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Username;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Credentials {
    pub id: Uuid,
    pub username: Username,
    pub active: bool,
    pub account_type: AccountType,
    pub access_token: String,
    pub refresh_token: String,
    pub expires: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    Offline,
    Microsoft,
}
