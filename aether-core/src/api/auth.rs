use uuid::Uuid;

use crate::{
    core::LauncherState,
    features::auth::{self, Account, FsCredentialsStorage},
};

#[tracing::instrument]
pub async fn get_accounts() -> crate::Result<Vec<Account>> {
    auth::get_accounts(&FsCredentialsStorage::new(LauncherState::get().await?)).await
}

#[tracing::instrument]
pub async fn create_offline_account(username: &str) -> crate::Result<Uuid> {
    auth::create_offline_account(
        &FsCredentialsStorage::new(LauncherState::get().await?),
        username,
    )
    .await
}

#[tracing::instrument]
pub async fn change_account(id: &Uuid) -> crate::Result<()> {
    auth::set_active_account(&FsCredentialsStorage::new(LauncherState::get().await?), id).await
}

#[tracing::instrument]
pub async fn logout(id: &Uuid) -> crate::Result<()> {
    auth::logout(&FsCredentialsStorage::new(LauncherState::get().await?), id).await
}
