use uuid::Uuid;

use crate::{core::LauncherState, features::auth::CredentialsStorage};

pub async fn set_active_account<S>(
    state: &LauncherState,
    storage: &S,
    id: &Uuid,
) -> crate::Result<()>
where
    S: CredentialsStorage + ?Sized,
{
    storage.set_active(state, id).await
}
