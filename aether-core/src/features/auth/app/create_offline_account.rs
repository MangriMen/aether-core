use chrono::Utc;
use uuid::Uuid;

use crate::features::auth::{Credentials, CredentialsStorage};

pub async fn create_offline_account<S>(storage: &S, username: &str) -> crate::Result<Uuid>
where
    S: CredentialsStorage + ?Sized,
{
    storage
        .upsert(&Credentials {
            id: Uuid::new_v4(),
            username: username.to_string(),
            access_token: String::new(),
            refresh_token: String::new(),
            expires: Utc::now(),
            active: true,
        })
        .await
}
