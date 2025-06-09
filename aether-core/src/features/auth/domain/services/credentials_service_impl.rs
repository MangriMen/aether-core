use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::features::auth::{AuthError, Credentials, CredentialsService, CredentialsStorage};

pub struct CredentialsServiceImpl<CS: CredentialsStorage> {
    credentials_storage: Arc<CS>,
}

impl<CS: CredentialsStorage> CredentialsServiceImpl<CS> {
    pub fn new(credentials_storage: Arc<CS>) -> Self {
        Self {
            credentials_storage,
        }
    }
}

#[async_trait]
impl<CS: CredentialsStorage> CredentialsService for CredentialsServiceImpl<CS> {
    async fn list(&self) -> Result<Vec<Credentials>, AuthError> {
        self.credentials_storage.list().await
    }

    async fn get(&self, id: Uuid) -> Result<Credentials, AuthError> {
        self.credentials_storage.get(id).await
    }

    async fn upsert(&self, credentials: Credentials) -> Result<Credentials, AuthError> {
        if credentials.active {
            self.credentials_storage.deactivate_all().await?;
        }
        self.credentials_storage.upsert(credentials.clone()).await?;
        Ok(credentials)
    }

    async fn remove(&self, id: Uuid) -> Result<(), AuthError> {
        let was_active = self.credentials_storage.get(id).await?.active;
        self.credentials_storage.remove(id).await?;

        if was_active {
            // Set the first one as active
            if let Some(first) = self.credentials_storage.list().await?.first() {
                let mut new_active = first.clone();
                new_active.active = true;
                self.credentials_storage.upsert(new_active).await?;
            }
        }

        Ok(())
    }

    async fn get_active(&self) -> Result<Credentials, AuthError> {
        self.credentials_storage.get_active().await
    }

    async fn set_active(&self, id: Uuid) -> Result<Credentials, AuthError> {
        let credentials = self.credentials_storage.get(id).await?;

        if credentials.active {
            return Ok(credentials);
        }

        self.credentials_storage.deactivate_all().await?;
        self.credentials_storage.set_active(id).await
    }
}
