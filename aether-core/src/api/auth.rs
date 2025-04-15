use uuid::Uuid;

use crate::{
    core::LauncherState,
    features::auth::{self, Account, FsCredentialsStorage},
};

async fn get_storage() -> crate::Result<FsCredentialsStorage> {
    Ok(FsCredentialsStorage::new(
        &LauncherState::get().await?.locations.settings_dir,
    ))
}

#[tracing::instrument]
pub async fn get_accounts() -> crate::Result<Vec<Account>> {
    auth::get_accounts(&get_storage().await?).await
}

#[tracing::instrument]
pub async fn create_offline_account(username: &str) -> crate::Result<Uuid> {
    auth::create_offline_account(&get_storage().await?, username).await
}

#[tracing::instrument]
pub async fn change_account(id: &Uuid) -> crate::Result<()> {
    auth::set_active_account(&get_storage().await?, id).await
}

#[tracing::instrument]
pub async fn logout(id: &Uuid) -> crate::Result<()> {
    auth::logout(&get_storage().await?, id).await
}
