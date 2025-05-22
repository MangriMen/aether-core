use uuid::Uuid;

use crate::{
    core::domain::LazyLocator,
    features::auth::{
        Account, CreateOfflineAccountUseCase, GetAccountsUseCase, LogoutUseCase,
        SetActiveAccountUseCase,
    },
};

#[tracing::instrument]
pub async fn create_offline_account(username: String) -> crate::Result<Uuid> {
    let lazy_locator = LazyLocator::get().await?;

    CreateOfflineAccountUseCase::new(lazy_locator.get_credentials_storage().await)
        .execute(username)
        .await
}

#[tracing::instrument]
pub async fn get_accounts() -> crate::Result<Vec<Account>> {
    let lazy_locator = LazyLocator::get().await?;

    GetAccountsUseCase::new(lazy_locator.get_credentials_storage().await)
        .execute()
        .await
}

#[tracing::instrument]
pub async fn change_account(account_id: Uuid) -> crate::Result<()> {
    let lazy_locator = LazyLocator::get().await?;

    SetActiveAccountUseCase::new(lazy_locator.get_credentials_storage().await)
        .execute(account_id)
        .await
}

#[tracing::instrument]
pub async fn logout(account_id: Uuid) -> crate::Result<()> {
    let lazy_locator = LazyLocator::get().await?;

    LogoutUseCase::new(lazy_locator.get_credentials_storage().await)
        .execute(account_id)
        .await
}
