use async_trait::async_trait;

use crate::features::java::Java;

#[async_trait]
pub trait JavaStorage {
    async fn list(&self) -> crate::Result<Vec<Java>>;
    async fn get(&self, version: u32) -> crate::Result<Option<Java>>;
    async fn upsert(&self, java: &Java) -> crate::Result<()>;
}
