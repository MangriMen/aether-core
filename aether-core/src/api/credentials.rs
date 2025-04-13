use uuid::Uuid;

use crate::{
    core::LauncherState,
    features::auth::{self, Account, FsCredentialsStorage},
};

#[tracing::instrument]
pub async fn get_accounts() -> crate::Result<Vec<Account>> {
    let state = LauncherState::get().await?;
    auth::get_accounts(&state, &FsCredentialsStorage).await
}

#[tracing::instrument]
pub async fn create_offline_account(username: &str) -> crate::Result<Uuid> {
    let state = LauncherState::get().await?;
    auth::create_offline_account(&state, &FsCredentialsStorage, username).await
}

#[tracing::instrument]
pub async fn change_account(id: &Uuid) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    auth::set_active_account(&state, &FsCredentialsStorage, id).await
}

#[tracing::instrument]
pub async fn logout(id: &Uuid) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    auth::logout(&state, &FsCredentialsStorage, id).await
}
