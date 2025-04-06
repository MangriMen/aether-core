use uuid::Uuid;

use crate::{
    core::LauncherState,
    features::{
        self,
        auth::{credentials_storage::CredentialsStorage, Account, FsCredentialsStorage},
    },
};

#[tracing::instrument]
pub async fn get_accounts() -> crate::Result<Vec<Account>> {
    let state = LauncherState::get().await?;
    let credentials = FsCredentialsStorage.get_all(&state).await?;
    Ok(credentials.into_iter().map(Account::from).collect())
}

#[tracing::instrument]
pub async fn create_offline_account(username: &str) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    features::auth::create_offline_account(&state, username).await
}

#[tracing::instrument]
pub async fn change_account(id: &Uuid) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    FsCredentialsStorage.set_active(&state, id).await?;
    Ok(())
}

#[tracing::instrument]
pub async fn logout(id: &Uuid) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    FsCredentialsStorage.remove(&state, id).await?;
    Ok(())
}
