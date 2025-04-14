use uuid::Uuid;

use crate::features::auth::CredentialsStorage;

pub async fn logout<S>(storage: &S, id: &Uuid) -> crate::Result<()>
where
    S: CredentialsStorage + ?Sized,
{
    storage.remove(id).await
}
