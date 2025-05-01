use std::path::Path;

use async_trait::async_trait;

use crate::features::java::Java;

#[async_trait]
pub trait JavaInstallationService {
    async fn locate_java(&self, path: &Path) -> Option<Java>;
}
