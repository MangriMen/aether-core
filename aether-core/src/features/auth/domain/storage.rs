use async_trait::async_trait;
use uuid::Uuid;

use crate::core::LauncherState;

use super::Credentials;

#[async_trait]
pub trait CredentialsStorage {
    async fn get(&self, state: &LauncherState, id: &Uuid) -> crate::Result<Credentials>;
    async fn get_active(&self, state: &LauncherState) -> crate::Result<Option<Credentials>>;
    async fn get_all(&self, state: &LauncherState) -> crate::Result<Vec<Credentials>>;

    async fn upsert(&self, state: &LauncherState, credentials: &Credentials)
        -> crate::Result<Uuid>;
    async fn set_active(&self, state: &LauncherState, id: &Uuid) -> crate::Result<()>;
    async fn upsert_all(
        &self,
        state: &LauncherState,
        credentials: Vec<Credentials>,
    ) -> crate::Result<()>;

    async fn remove(&self, state: &LauncherState, id: &Uuid) -> crate::Result<()>;
}
