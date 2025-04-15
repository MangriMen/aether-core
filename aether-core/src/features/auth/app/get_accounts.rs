use crate::features::auth::{Account, CredentialsStorage};

pub async fn get_accounts<S>(storage: &S) -> crate::Result<Vec<Account>>
where
    S: CredentialsStorage + ?Sized,
{
    let credentials = storage.list().await?;
    Ok(credentials.into_iter().map(Account::from).collect())
}
