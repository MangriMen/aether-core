use std::{path::PathBuf, sync::Arc};

use crate::features::java::{ports::JreProvider, JavaError};

pub struct InstallJreUseCase<JP: JreProvider> {
    provider: Arc<JP>,
}

impl<JP: JreProvider> InstallJreUseCase<JP> {
    pub fn new(provider: Arc<JP>) -> Self {
        Self { provider }
    }

    pub async fn execute(&self, version: u32, install_dir: PathBuf) -> Result<PathBuf, JavaError> {
        self.provider.install(version, &install_dir).await
    }
}
