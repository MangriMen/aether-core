use uuid::Uuid;

use crate::features::auth::CredentialsStorage;

pub async fn set_active_account<S>(storage: &S, id: &Uuid) -> crate::Result<()>
where
    S: CredentialsStorage + ?Sized,
{
    storage.set_active(id).await
}
