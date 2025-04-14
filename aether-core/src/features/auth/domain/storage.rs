use async_trait::async_trait;
use uuid::Uuid;

use super::Credentials;

#[async_trait]
pub trait CredentialsStorage {
    async fn get(&self, id: &Uuid) -> crate::Result<Credentials>;
    async fn get_active(&self) -> crate::Result<Option<Credentials>>;
    async fn get_all(&self) -> crate::Result<Vec<Credentials>>;

    async fn upsert(&self, credentials: &Credentials) -> crate::Result<Uuid>;
    async fn set_active(&self, id: &Uuid) -> crate::Result<()>;
    async fn upsert_all(&self, credentials: Vec<Credentials>) -> crate::Result<()>;

    async fn remove(&self, id: &Uuid) -> crate::Result<()>;
}
