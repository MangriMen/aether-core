use uuid::Uuid;

use crate::state::{Account, Credentials};

#[tracing::instrument]
pub async fn get_accounts() -> crate::Result<Vec<Account>> {
    let state = crate::state::LauncherState::get().await?;

    let credentials = Credentials::get(&state).await?;

    Ok(credentials.into_iter().map(Account::from).collect())
}

#[tracing::instrument]
pub async fn create_offline_account(username: &str) -> crate::Result<()> {
    let state = crate::state::LauncherState::get().await?;

    Credentials::create_offline_account(&state, username).await
}

#[tracing::instrument]
pub async fn change_account(id: &Uuid) -> crate::Result<()> {
    let state = crate::state::LauncherState::get().await?;

    Credentials::set_active(&state, id).await?;

    Ok(())
}

#[tracing::instrument]
pub async fn logout(id: &Uuid) -> crate::Result<()> {
    let state = crate::state::LauncherState::get().await?;

    Credentials::remove(&state, id).await?;

    Ok(())
}
