use crate::{
    core::LauncherState,
    features::auth::{Account, CredentialsStorage},
};

pub async fn get_accounts<S>(state: &LauncherState, storage: &S) -> crate::Result<Vec<Account>>
where
    S: CredentialsStorage + ?Sized,
{
    let credentials = storage.get_all(state).await?;
    Ok(credentials.into_iter().map(Account::from).collect())
}
