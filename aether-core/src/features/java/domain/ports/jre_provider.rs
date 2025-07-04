use std::path::{Path, PathBuf};

use async_trait::async_trait;

use crate::features::java::JavaError;

#[async_trait]
pub trait JreProvider: Send + Sync {
    async fn install(&self, version: u32, install_dir: &Path) -> Result<PathBuf, JavaError>;
}
