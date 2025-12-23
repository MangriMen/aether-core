use async_trait::async_trait;
use uuid::Uuid;

use crate::features::auth::{AuthApplicationError, Credentials};

#[async_trait]
pub trait CredentialsStorage: Send + Sync {
    async fn list(&self) -> Result<Vec<Credentials>, AuthApplicationError>;
    async fn get(&self, id: Uuid) -> Result<Credentials, AuthApplicationError>;
    async fn upsert(&self, credentials: Credentials) -> Result<Credentials, AuthApplicationError>;
    async fn remove(&self, id: Uuid) -> Result<(), AuthApplicationError>;

    async fn get_first(&self) -> Result<Credentials, AuthApplicationError>;

    async fn get_active(&self) -> Result<Credentials, AuthApplicationError>;
    async fn set_active(&self, id: Uuid) -> Result<Credentials, AuthApplicationError>;

    async fn deactivate_all(&self) -> Result<(), AuthApplicationError>;
}
