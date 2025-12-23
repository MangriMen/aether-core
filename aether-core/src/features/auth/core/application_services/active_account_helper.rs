use crate::features::auth::{AuthApplicationError, Credentials, CredentialsStorage};

pub struct ActiveAccountHelper;

impl ActiveAccountHelper {
    pub async fn ensure_active(
        storage: &dyn CredentialsStorage,
    ) -> Result<Credentials, AuthApplicationError> {
        if let Ok(active) = storage.get_active().await {
            return Ok(active);
        }

        let first = storage.get_first().await?;
        let new_first = storage.set_active(first.id).await?;
        Ok(new_first)
    }

    pub async fn set_active(
        storage: &dyn CredentialsStorage,
        id: uuid::Uuid,
    ) -> Result<Credentials, AuthApplicationError> {
        let active_result = storage.get_active().await;

        if let Ok(active) = active_result {
            if active.id == id {
                return Ok(active);
            }
        }

        storage.deactivate_all().await?;
        let account = storage.set_active(id).await?;
        Ok(account)
    }
}
