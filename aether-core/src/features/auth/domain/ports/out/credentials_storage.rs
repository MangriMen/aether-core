use async_trait::async_trait;
use uuid::Uuid;

use crate::features::auth::{AuthError, Credentials};

#[async_trait]
pub trait CredentialsStorage: Send + Sync {
    async fn list(&self) -> Result<Vec<Credentials>, AuthError>;
    async fn get(&self, id: Uuid) -> Result<Credentials, AuthError>;
    async fn upsert(&self, credentials: Credentials) -> Result<Credentials, AuthError>;
    async fn remove(&self, id: Uuid) -> Result<(), AuthError>;

    async fn get_active(&self) -> Result<Credentials, AuthError>;
    async fn set_active(&self, id: Uuid) -> Result<Credentials, AuthError>;

    async fn deactivate_all(&self) -> Result<(), AuthError>;
}
