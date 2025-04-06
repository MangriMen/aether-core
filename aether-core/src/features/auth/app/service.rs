use chrono::Utc;
use uuid::Uuid;

use crate::{
    core::LauncherState,
    features::auth::{credentials_storage::CredentialsStorage, Credentials, FsCredentialsStorage},
};

pub async fn create_offline_account(state: &LauncherState, username: &str) -> crate::Result<()> {
    let new_credentials_id = Uuid::new_v4();

    let new_credentials = Credentials {
        id: new_credentials_id,
        username: username.to_string(),
        access_token: String::new(),
        refresh_token: String::new(),
        expires: Utc::now(),
        active: true,
    };

    FsCredentialsStorage.upsert(state, &new_credentials).await?;

    Ok(())
}
