use async_trait::async_trait;

use crate::features::java::{Java, JavaError};

#[async_trait]
pub trait JavaStorage: Send + Sync {
    async fn list(&self) -> Result<Vec<Java>, JavaError>;
    async fn get(&self, version: u32) -> Result<Option<Java>, JavaError>;
    async fn upsert(&self, java: &Java) -> Result<(), JavaError>;
}
