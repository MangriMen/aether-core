use std::path::Path;

use async_trait::async_trait;

use crate::{core::LauncherState, features::java::Java};

#[async_trait]
pub trait JavaStorage {
    async fn create_from_path(&self, path: &Path) -> crate::Result<Java>;

    async fn get(&self, state: &LauncherState, version: u32) -> crate::Result<Option<Java>>;
    async fn upsert(&self, state: &LauncherState, java: &Java) -> crate::Result<()>;
}
