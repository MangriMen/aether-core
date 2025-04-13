use chrono::Utc;
use uuid::Uuid;

use crate::{
    core::LauncherState,
    features::auth::{storage::CredentialsStorage, Credentials},
};

pub async fn create_offline_account<S>(
    state: &LauncherState,
    storage: &S,
    username: &str,
) -> crate::Result<Uuid>
where
    S: CredentialsStorage + ?Sized,
{
    storage
        .upsert(
            state,
            &Credentials {
                id: Uuid::new_v4(),
                username: username.to_string(),
                access_token: String::new(),
                refresh_token: String::new(),
                expires: Utc::now(),
                active: true,
            },
        )
        .await
}
