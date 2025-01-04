use uuid::Uuid;

use crate::state::{Account, AccountState, Credentials};

pub async fn get_accounts() -> crate::Result<AccountState> {
    let state = crate::state::LauncherState::get().await?;

    let credentials = Credentials::get_state(&state).await?;

    Ok(AccountState {
        default: credentials.default,
        accounts: credentials
            .credentials
            .iter()
            .map(|cred| Account::from(cred.clone()))
            .collect(),
    })
}

pub async fn create_offline_account(username: &str) -> crate::Result<()> {
    let state = crate::state::LauncherState::get().await?;

    Credentials::create_offline_account(&state, username).await
}

pub async fn change_account(id: &Uuid) -> crate::Result<()> {
    let state = crate::state::LauncherState::get().await?;

    Credentials::change_default(&state, &id).await?;

    Ok(())
}

pub async fn logout(id: &Uuid) -> crate::Result<()> {
    let state = crate::state::LauncherState::get().await?;

    Credentials::remove(&state, &id).await?;

    Ok(())
}
