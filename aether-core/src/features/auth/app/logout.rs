use uuid::Uuid;

use crate::{core::LauncherState, features::auth::CredentialsStorage};

pub async fn logout<S>(state: &LauncherState, storage: &S, id: &Uuid) -> crate::Result<()>
where
    S: CredentialsStorage + ?Sized,
{
    storage.remove(state, id).await
}
